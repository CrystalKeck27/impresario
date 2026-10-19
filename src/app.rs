use leptos::{either::Either, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Instant,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub is_directory: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct FileTag {
    key: String,
    value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcoustidLookup {
    fingerprint: String,
    best_match: Option<AcoustidMatch>,
    lookup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcoustidMatch {
    score: Option<f64>,
    acoustid_id: Option<String>,
    recording_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FingerprintJobStatus {
    pub job_id: String,
    pub total: usize,
    pub completed: usize,
    pub stored: usize,
    pub skipped_existing: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub is_complete: bool,
}

#[cfg(feature = "ssr")]
struct FingerprintJobEntry {
    status: FingerprintJobStatus,
    updated_at: Instant,
}

#[cfg(feature = "ssr")]
static FINGERPRINT_JOBS: OnceLock<Mutex<HashMap<String, FingerprintJobEntry>>> = OnceLock::new();

#[cfg(feature = "ssr")]
fn fingerprint_jobs() -> &'static Mutex<HashMap<String, FingerprintJobEntry>> {
    FINGERPRINT_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(feature = "hydrate")]
async fn fingerprint_poll_delay() {
    gloo_timers::future::TimeoutFuture::new(500).await;
}

#[cfg(all(not(feature = "hydrate"), feature = "ssr"))]
async fn fingerprint_poll_delay() {}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/impresario.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("files") view=FilesPage/>
                    <Route path=StaticSegment("fingerprint") view=FingerprintPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <a href="/files"><button>"Files"</button></a>
        <a class="home-page__link" href="/fingerprint">"Fingerprint a directory"</a>
    }
}

#[component]
fn FingerprintPage() -> impl IntoView {
    use leptos::task::spawn_local;

    let draft_path = RwSignal::new(".".to_string());
    let job_status = RwSignal::new(None::<FingerprintJobStatus>);
    let running = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if running.get_untracked() {
            return;
        }

        let directory = draft_path.get_untracked();
        job_status.set(None);
        error.set(None);
        running.set(true);

        spawn_local(async move {
            match start_fingerprint_job(directory).await {
                Ok(mut status) => {
                    job_status.set(Some(status.clone()));
                    while !status.is_complete {
                        fingerprint_poll_delay().await;
                        match get_fingerprint_job_status(status.job_id.clone()).await {
                            Ok(updated) => {
                                status = updated;
                                job_status.set(Some(status.clone()));
                            }
                            Err(fetch_error) => {
                                error.set(Some(fetch_error.to_string()));
                                running.set(false);
                                return;
                            }
                        }
                    }
                }
                Err(start_error) => error.set(Some(start_error.to_string())),
            }
            running.set(false);
        });
    };

    view! {
        <section class="fingerprint-page">
            <header class="fingerprint-page__header">
                <h1>"Fingerprint a directory"</h1>
                <p>"Recursively fingerprint files with fpcalc and save new results to PostgreSQL."</p>
            </header>
            <form class="fingerprint-form" on:submit=submit>
                <label class="fingerprint-form__label" for="fingerprint-directory">
                    "Directory path"
                </label>
                <div class="fingerprint-form__controls">
                    <input
                        id="fingerprint-directory"
                        class="fingerprint-form__input"
                        type="text"
                        prop:value=move || draft_path.get()
                        placeholder="Enter a directory path"
                        on:input=move |event| draft_path.set(event_target_value(&event))
                    />
                    <button class="fingerprint-form__button" type="submit" disabled=move || running.get()>
                        {move || if running.get() { "Fingerprinting…" } else { "Start fingerprinting" }}
                    </button>
                </div>
            </form>
            {move || error.get().map(|message| view! {
                <p class="fingerprint-page__error" role="alert">{message}</p>
            })}
            {move || match job_status.get() {
                None => ().into_any(),
                Some(status) => {
                    let progress_max = status.total.max(1);
                    let percentage = status
                        .completed
                        .saturating_mul(100)
                        .checked_div(status.total)
                        .unwrap_or(100);
                    view! {
                        <section class="fingerprint-progress" aria-live="polite">
                            <div class="fingerprint-progress__heading">
                                <h2>{if status.is_complete { "Fingerprinting complete" } else { "Fingerprinting in progress" }}</h2>
                                <span>{format!("{percentage}%")}</span>
                            </div>
                            <progress
                                class="fingerprint-progress__bar"
                                max=progress_max
                                value=status.completed
                                aria-label="Fingerprinting progress"
                            ></progress>
                            <p class="fingerprint-progress__summary">
                                {format!("{} of {} processed · {} saved · {} failed · {} already fingerprinted", status.completed, status.total, status.stored, status.failed, status.skipped_existing)}
                            </p>
                            {if status.errors.is_empty() {
                                ().into_any()
                            } else {
                                view! {
                                    <details class="fingerprint-progress__errors">
                                        <summary>{format!("{} file(s) failed; showing {} error details", status.failed, status.errors.len())}</summary>
                                        <ul>{status.errors.into_iter().map(|item| view! { <li>{item}</li> }).collect::<Vec<_>>()}</ul>
                                    </details>
                                }.into_any()
                            }}
                        </section>
                    }.into_any()
                }
            }}
        </section>
    }
}

/// Renders the files page of your application.
#[component]
fn FilesPage() -> impl IntoView {
    let draft_path = RwSignal::new(".".to_string());
    let root_path = RwSignal::new(".".to_string());

    let submit_root = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        root_path.set(draft_path.get());
    };

    view! {
        <section class="files-page">
            <header class="files-page__header">
                <h1>"Files Page"</h1>
                <form class="root-path-form" on:submit=submit_root>
                    <label class="root-path-form__label" for="root-path">
                        "Explorer root"
                    </label>
                    <div class="root-path-form__controls">
                        <input
                            id="root-path"
                            class="root-path-form__input"
                            type="text"
                            prop:value=move || draft_path.get()
                            placeholder="Enter a directory path"
                            on:input=move |ev| draft_path.set(event_target_value(&ev))
                        />
                        <button class="root-path-form__button" type="submit">
                            "Load"
                        </button>
                    </div>
                </form>
            </header>

            <ExplorerView path=root_path />
        </section>
    }
}

#[component]
fn ExplorerView(path: RwSignal<String>) -> impl IntoView {
    let selected_file = RwSignal::new(None::<String>);
    let entities = Resource::new(
        move || path.get(),
        |path| async move {
            if path.trim().is_empty() {
                Ok(vec![])
            } else {
                fetch_files_in_directory(path).await
            }
        },
    );

    view! {
        <div class="explorer-view">
            <div class="explorer-view__layout">
                <section class="explorer-view__browser">
                    <p class="explorer-view__root">
                        <strong>"Current root: "</strong>
                        {move || path.get()}
                    </p>
                    <table>
                        <thead>
                            <tr>
                                <th>"Name"</th>
                                <th>"Type"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Transition fallback=move || view! {
                                <StatusRow message="Loading files..." />
                            }>
                                {move || {
                                    let current_path = path.get();

                                    match entities.get() {
                                        None => Either::Left(view! {
                                            <StatusRow message="Loading files..." />
                                        }),
                                        Some(Err(error)) => Either::Left(view! {
                                            <StatusRow message=format!("Error loading files: {error}") />
                                        }),
                                        Some(Ok(_)) if current_path.trim().is_empty() => {
                                            Either::Left(view! {
                                                <StatusRow message="Enter a directory path to begin." />
                                            })
                                        }
                                        Some(Ok(entries)) if entries.is_empty() => Either::Left(view! {
                                            <StatusRow message="No files found." />
                                        }),
                                        Some(Ok(entries)) => Either::Right(
                                            entries
                                                .into_iter()
                                                .map(|entry| {
                                                    let item_path = format!("{}/{}", current_path, entry.name);
                                                    let item_name = entry.name.clone();
                                                    let item_name_clone = item_name.clone();
                                                    if entry.is_directory {
                                                        Either::Left(view! {
                                                            <DirectoryItem
                                                                name=item_name_clone
                                                                path=item_path
                                                                depth=1
                                                                selected_file
                                                            />
                                                        })
                                                    } else {
                                                        Either::Right(view! {
                                                            <FileItem
                                                                name=item_name
                                                                path=item_path
                                                                depth=1
                                                                selected_file
                                                            />
                                                        })
                                                    }
                                                })
                                                .collect::<Vec<_>>(),
                                        ),
                                    }
                                }}
                            </Transition>
                        </tbody>
                    </table>
                </section>

                <FileTagsPanel selected_file />
            </div>
        </div>
    }
}

#[component]
fn StatusRow(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <tr>
            <td colspan="2" class="explorer-view__status">{message}</td>
        </tr>
    }
}

#[component]
fn FileItem(
    name: String,
    path: String,
    depth: usize,
    selected_file: RwSignal<Option<String>>,
) -> impl IntoView {
    let style_string = format!("margin-left: {}px;", depth * 20);
    let path_for_click = path.clone();
    let path_for_selected = path.clone();

    view! {
        <tr
            class:selected=move || selected_file.get().as_deref() == Some(path_for_selected.as_str())
            on:click=move |_| selected_file.set(Some(path_for_click.clone()))
        >
            <td><div style=style_string>{name}</div></td>
            <td>"File"</td>
        </tr>
    }
}

#[component]
fn DirectoryItem(
    name: String,
    path: String,
    depth: usize,
    selected_file: RwSignal<Option<String>>,
) -> impl IntoView {
    let path_clone = path.clone();
    let is_expanded = RwSignal::new(false);
    let style_string = format!(
        "width: {}px; display: inline-block; float: left; height: 1px; text-align: right;",
        depth * 20
    );

    view! {
        <tr on:click=move |_| {
            is_expanded.set(!is_expanded.get());
        }>
            <td style=""><div style={style_string}>{move || if is_expanded.get() { "▼" } else { "►" }}</div><div>{name}</div></td>
            <td>{move || if is_expanded.get() { "Expanded" } else { "Collapsed" }}</td>
        </tr>
        <Show when=move || is_expanded.get()>
            <Subdirectory path=&path_clone depth=depth + 1 selected_file />
        </Show>
    }
}

#[component]
fn Subdirectory<'a>(
    path: &'a str,
    depth: usize,
    selected_file: RwSignal<Option<String>>,
) -> impl IntoView {
    let path_clone = path.to_string();
    let entries = Resource::new(
        move || true,
        move |is_ex| {
            let p = path_clone.clone();
            async move {
                if is_ex {
                    fetch_files_in_directory(p.to_string())
                        .await
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            }
        },
    );

    let suspense = move || {
        let path_clone = path.to_string();
        Suspend::new(async move {
            let entries = entries.await;
            if entries.is_empty() {
                Either::Left(())
            } else {
                Either::Right(
                    entries
                        .into_iter()
                        .map(|entry| {
                            let item_path = format!("{}/{}", path_clone, entry.name);
                            let item_name = entry.name.clone();
                            let item_name_clone = item_name.clone();
                            if entry.is_directory {
                                Either::Left(view! {
                                    <DirectoryItem
                                        name=item_name_clone
                                        path=item_path
                                        depth=depth
                                        selected_file
                                    />
                                })
                            } else {
                                Either::Right(view! {
                                    <FileItem
                                        name=item_name
                                        path=item_path
                                        depth=depth
                                        selected_file
                                    />
                                })
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            }
        })
    };

    view! {
        {suspense()}
    }
    .into_any()
}

#[component]
fn FileTagsPanel(selected_file: RwSignal<Option<String>>) -> impl IntoView {
    let acoustid_match = Resource::new(
        move || selected_file.get(),
        |selected_path| async move {
            match selected_path {
                Some(path) => fetch_acoustid_match(path).await.map(Some),
                None => Ok(None),
            }
        },
    );
    let tags = Resource::new(
        move || selected_file.get(),
        |selected_path| async move {
            match selected_path {
                Some(path) => fetch_file_content(path).await.map(|items| {
                    items
                        .into_iter()
                        .map(|(key, value)| FileTag { key, value })
                        .collect::<Vec<_>>()
                }),
                None => Ok(vec![]),
            }
        },
    );

    view! {
        <aside class="tags-panel">
            <div class="tags-panel__header">
                <p class="tags-panel__eyebrow">"Selected file"</p>
                <h2>"Tags"</h2>
                <p class="tags-panel__path">
                    {move || {
                        selected_file
                            .get()
                            .unwrap_or_else(|| "Choose a file in the explorer to inspect its tags.".to_string())
                    }}
                </p>
            </div>

            <section class="tags-panel__fingerprint">
                <h3>"AcoustID fingerprint"</h3>
                <Suspense fallback=move || view! {
                    <p class="tags-panel__status">"Calculating fingerprint..."</p>
                }>
                    {move || match selected_file.get() {
                        None => view! {
                            <p class="tags-panel__status">"Choose a file to calculate its fingerprint."</p>
                        }.into_any(),
                        Some(_) => match acoustid_match.get() {
                            None => view! {
                                <p class="tags-panel__status">"Calculating fingerprint..."</p>
                            }.into_any(),
                            Some(Err(error)) => view! {
                                <p class="tags-panel__status">{format!("Could not calculate fingerprint: {error}")}</p>
                            }.into_any(),
                            Some(Ok(None)) => view! {
                                <p class="tags-panel__status">"Choose a file to calculate its fingerprint."</p>
                            }.into_any(),
                            Some(Ok(Some(lookup))) => view! {
                                <pre class="tags-panel__fingerprint-value">{lookup.fingerprint}</pre>
                            }.into_any(),
                        },
                    }}
                </Suspense>
            </section>

            <section class="tags-panel__match">
                <h3>"Closest AcoustID match"</h3>
                <Suspense fallback=move || view! {
                    <p class="tags-panel__status">"Looking up this recording..."</p>
                }>
                    {move || match selected_file.get() {
                        None => view! {
                            <p class="tags-panel__status">"Choose a file to look up its closest match."</p>
                        }.into_any(),
                        Some(_) => match acoustid_match.get() {
                            None => view! {
                                <p class="tags-panel__status">"Looking up this recording..."</p>
                            }.into_any(),
                            Some(Err(error)) => view! {
                                <p class="tags-panel__status">{format!("AcoustID lookup failed: {error}")}</p>
                            }.into_any(),
                            Some(Ok(None)) => view! {
                                <p class="tags-panel__status">"No AcoustID match was found."</p>
                            }.into_any(),
                            Some(Ok(Some(lookup))) => match lookup.lookup_error {
                                Some(error) => view! {
                                    <p class="tags-panel__status">{format!("AcoustID lookup failed: {error}")}</p>
                                }.into_any(),
                                None => match lookup.best_match {
                                None => view! {
                                    <p class="tags-panel__status">"No AcoustID match was found."</p>
                                }.into_any(),
                                Some(result) => {
                                    let score = result.score
                                        .map(|score| format!("Match confidence: {:.0}%", score * 100.0));
                                    let has_acoustid_id = result.acoustid_id.is_some();
                                    let acoustid_link = result.acoustid_id.map(|id| view! {
                                        <a
                                            class="tags-panel__match-link"
                                            href=format!("https://acoustid.org/track/{id}")
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >"Open AcoustID match"</a>
                                    });
                                    let musicbrainz_link = result.recording_id.map(|id| view! {
                                        <a
                                            class="tags-panel__match-link"
                                            href=format!("https://musicbrainz.org/recording/{id}")
                                            target="_blank"
                                            rel="noopener noreferrer"
                                        >"Open MusicBrainz recording"</a>
                                    });
                                    view! {
                                        <div class="tags-panel__match-result">
                                            {score.map(|score| view! {
                                                <p class="tags-panel__match-score">{score}</p>
                                            })}
                                            {acoustid_link}
                                            {musicbrainz_link}
                                            {if !has_acoustid_id {
                                                Some(view! { <p class="tags-panel__status">"The match did not include an AcoustID track ID."</p> })
                                            } else {
                                                None
                                            }}
                                        </div>
                                    }.into_any()
                                },
                                },
                            },
                        },
                    }}
                </Suspense>
            </section>

            <h3>"Tags"</h3>
            <Suspense fallback=move || view! {
                <p class="tags-panel__status">"Loading tags..."</p>
            }>
                {move || match selected_file.get() {
                    None => view! {
                        <p class="tags-panel__status">"Choose a file in the explorer to inspect its tags."</p>
                    }.into_any(),
                    Some(_) => match tags.get() {
                        None => view! {
                            <p class="tags-panel__status">"Loading tags..."</p>
                        }.into_any(),
                        Some(Err(error)) => view! {
                            <p class="tags-panel__status">{format!("Could not read tags: {error}")}</p>
                        }.into_any(),
                        Some(Ok(file_tags)) if file_tags.is_empty() => view! {
                            <p class="tags-panel__status">"No tags found for this file."</p>
                        }.into_any(),
                        Some(Ok(file_tags)) => view! {
                            <dl class="tags-panel__list">
                                {file_tags
                                    .into_iter()
                                    .map(|tag| {
                                        view! {
                                            <div class="tags-panel__item">
                                                <dt>{tag.key}</dt>
                                                <dd>{tag.value}</dd>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </dl>
                        }.into_any(),
                    },
                }}
            </Suspense>
        </aside>
    }
}

/// Fetches files and directories recursively and returns a list of file entries. This function is only available on the server side.
#[server]
pub async fn fetch_files_in_directory(
    path: String,
) -> Result<Vec<FileEntry>, ServerFnError<String>> {
    use std::fs;

    let entries = fs::read_dir(path.clone()).map_err(|e| {
        eprintln!("Failed to read directory '{}': {}", path, e);
        ServerFnError::ServerError(e.to_string())
    })?;
    let mut file_entries = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| {
            eprintln!("Failed to read entry in '{}': {}", path, e);
            ServerFnError::ServerError(e.to_string())
        })?;
        if let Some(file_name) = entry.file_name().to_str() {
            let is_directory = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);

            file_entries.push(FileEntry {
                name: file_name.to_string(),
                is_directory,
            });
        }
    }

    // Sort with directories first, then alphabetically
    file_entries.sort_by(|a, b| {
        if a.is_directory != b.is_directory {
            b.is_directory.cmp(&a.is_directory)
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(file_entries)
}

#[server]
pub async fn start_fingerprint_job(
    directory: String,
) -> Result<FingerprintJobStatus, ServerFnError<String>> {
    let (paths, skipped_existing, invalid_paths) = tokio::task::spawn_blocking(move || {
        let root = fs::canonicalize(&directory)
            .map_err(|error| format!("Could not access directory: {error}"))?;
        if !root.is_dir() {
            return Err("Fingerprint input must be a directory".to_string());
        }

        let mut discovered = Vec::new();
        collect_regular_files(&root, &mut discovered)
            .map_err(|error| format!("Could not scan directory: {error}"))?;

        let stored_paths: HashSet<String> = crate::database::existing_fingerprint_paths()
            .map_err(|error| format!("Could not check stored fingerprints: {error}"))?
            .into_iter()
            .collect();

        let (paths, skipped_existing, invalid_paths) =
            partition_existing_paths(discovered, &stored_paths);

        Ok::<_, String>((paths, skipped_existing, invalid_paths))
    })
    .await
    .map_err(|error| ServerFnError::ServerError(format!("Could not prepare scan: {error}")))?
    .map_err(ServerFnError::ServerError)?;

    let invalid_count = invalid_paths.len();
    let job_id = uuid::Uuid::new_v4().to_string();
    let status = FingerprintJobStatus {
        job_id: job_id.clone(),
        total: paths.len() + invalid_count,
        completed: invalid_count,
        stored: 0,
        skipped_existing,
        failed: invalid_count,
        errors: invalid_paths.into_iter().take(10).collect(),
        is_complete: paths.is_empty(),
    };

    {
        let mut jobs = fingerprint_jobs()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        jobs.retain(|_, entry| {
            !entry.status.is_complete || entry.updated_at.elapsed().as_secs() < 3600
        });
        jobs.insert(
            job_id.clone(),
            FingerprintJobEntry {
                status: status.clone(),
                updated_at: Instant::now(),
            },
        );
    }

    if !paths.is_empty() {
        tokio::spawn(run_fingerprint_job(job_id, paths));
    }

    Ok(status)
}

#[server]
pub async fn get_fingerprint_job_status(
    job_id: String,
) -> Result<FingerprintJobStatus, ServerFnError<String>> {
    let jobs = fingerprint_jobs()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    jobs.get(&job_id)
        .map(|entry| entry.status.clone())
        .ok_or_else(|| ServerFnError::ServerError("Fingerprint job was not found".to_string()))
}

#[cfg(feature = "ssr")]
fn collect_regular_files(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            collect_regular_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn partition_existing_paths(
    discovered: Vec<PathBuf>,
    stored_paths: &HashSet<String>,
) -> (Vec<String>, usize, Vec<String>) {
    let mut paths = Vec::new();
    let mut skipped_existing = 0;
    let mut invalid_paths = Vec::new();
    for path in discovered {
        if !is_audio_file(&path) {
            continue;
        }
        match path.to_str() {
            Some(path) if stored_paths.contains(path) => skipped_existing += 1,
            Some(path) => paths.push(path.to_string()),
            None => invalid_paths.push(format!(
                "A file path is not valid UTF-8: {}",
                path.display()
            )),
        }
    }
    paths.sort();
    (paths, skipped_existing, invalid_paths)
}

#[cfg(feature = "ssr")]
fn is_audio_file(path: &Path) -> bool {
    const AUDIO_EXTENSIONS: &[&str] = &[
        "aac", "aif", "aiff", "alac", "ape", "au", "caf", "dff", "dsf", "flac", "m4a", "m4b",
        "m4p", "mid", "midi", "mka", "mp3", "mpc", "oga", "ogg", "opus", "ra", "ram", "snd", "tta",
        "voc", "wav", "wave", "wma", "wv",
    ];

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            AUDIO_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

#[cfg(feature = "ssr")]
async fn run_fingerprint_job(job_id: String, paths: Vec<String>) {
    use tokio::task::JoinSet;

    let concurrency = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(2)
        .clamp(1, 8);
    let mut pending = paths.into_iter();
    let mut tasks = JoinSet::new();

    for _ in 0..concurrency {
        if let Some(path) = pending.next() {
            tasks.spawn(async move {
                let outcome = process_fingerprint_file(path.clone()).await;
                (path, outcome)
            });
        }
    }

    while let Some(result) = tasks.join_next().await {
        let outcome = match result {
            Ok((path, outcome)) => match outcome {
                Ok(stored) => Ok((path, stored)),
                Err(error) => Err(format!("{path}: {error}")),
            },
            Err(error) => Err(format!("Fingerprint worker failed: {error}")),
        };

        let mut jobs = fingerprint_jobs()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = jobs.get_mut(&job_id) {
            entry.status.completed += 1;
            match outcome {
                Ok((_, true)) => entry.status.stored += 1,
                Ok((_, false)) => entry.status.skipped_existing += 1,
                Err(message) => {
                    entry.status.failed += 1;
                    if entry.status.errors.len() < 10 {
                        entry.status.errors.push(message);
                    }
                }
            }
            entry.updated_at = Instant::now();
        }
        drop(jobs);

        if let Some(path) = pending.next() {
            tasks.spawn(async move {
                let outcome = process_fingerprint_file(path.clone()).await;
                (path, outcome)
            });
        }
    }

    let mut jobs = fingerprint_jobs()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = jobs.get_mut(&job_id) {
        entry.status.is_complete = true;
        entry.updated_at = Instant::now();
    }
}

#[cfg(feature = "ssr")]
async fn process_fingerprint_file(path: String) -> Result<bool, String> {
    let (fingerprint, duration) = calculate_fingerprint(Path::new(&path)).await?;
    if duration.round() > i32::MAX as f64 {
        return Err(format!("Duration is too long to store: {path}"));
    }
    let duration_seconds = duration.round() as i32;
    tokio::task::spawn_blocking(move || {
        crate::database::store_fingerprint(path, fingerprint.into_bytes(), duration_seconds)
    })
    .await
    .map_err(|error| format!("Database worker failed: {error}"))?
}

#[cfg(feature = "ssr")]
async fn calculate_fingerprint(path: &Path) -> Result<(String, f64), String> {
    use std::process::Stdio;
    use tokio::process::Command;

    let output = Command::new("fpcalc")
        .arg("-json")
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|error| format!("Could not run fpcalc: {error}"))?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            format!("fpcalc exited with status {}", output.status)
        } else {
            format!("fpcalc failed: {message}")
        });
    }

    parse_fpcalc_output(&output.stdout)
}

#[cfg(feature = "ssr")]
fn parse_fpcalc_output(output: &[u8]) -> Result<(String, f64), String> {
    #[derive(Deserialize)]
    struct FpcalcOutput {
        duration: f64,
        fingerprint: String,
    }

    let parsed: FpcalcOutput = serde_json::from_slice(output)
        .map_err(|error| format!("Could not parse fpcalc output: {error}"))?;
    if !parsed.duration.is_finite() || parsed.duration <= 0.0 || parsed.duration > u32::MAX as f64 {
        return Err("fpcalc returned an invalid duration".to_string());
    }
    Ok((parsed.fingerprint, parsed.duration))
}

#[server]
pub async fn fetch_file_content(
    path: String,
) -> Result<Vec<(String, String)>, ServerFnError<String>> {
    use lofty::file::TaggedFileExt as _;

    let tags = lofty::read_from_path(path);

    let mapped = tags.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let tag = match mapped.primary_tag() {
        Some(primary_tag) => primary_tag,
        // If the "primary" tag doesn't exist, we just grab the
        // first tag we can find. Realistically, a tag reader would likely
        // iterate through the tags to find a suitable one.
        None => mapped.first_tag().ok_or(ServerFnError::ServerError(
            "No tag found for file".to_string(),
        ))?,
    };

    Ok(tag
        .items()
        .map(|item| {
            (
                item.key()
                    .map_key(lofty::tag::TagType::VorbisComments)
                    .unwrap_or("N/A")
                    .to_string(),
                item.value()
                    .clone()
                    .into_string()
                    .unwrap_or("N/A".to_string()),
            )
        })
        .collect::<Vec<_>>())
}

#[server]
pub async fn fetch_acoustid_match(path: String) -> Result<AcoustidLookup, ServerFnError<String>> {
    use acoustid::{AcoustIDClient, Metadata};
    use std::fs;

    let path = fs::canonicalize(&path)
        .map_err(|e| ServerFnError::ServerError(format!("Could not access file: {e}")))?;
    let metadata = fs::metadata(&path)
        .map_err(|e| ServerFnError::ServerError(format!("Could not access file: {e}")))?;
    if !metadata.is_file() {
        return Err(ServerFnError::ServerError(
            "Fingerprint input must be a file".to_string(),
        ));
    }

    let (fingerprint, duration) = calculate_fingerprint(&path)
        .await
        .map_err(ServerFnError::ServerError)?;
    let application_key = std::env::var("ACOUSTID_APPLICATION_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty());
    let (best_match, lookup_error) = match application_key {
        None => (
            None,
            Some("ACOUSTID_APPLICATION_KEY is not set on the server".to_string()),
        ),
        Some(key) => match AcoustIDClient::with_application_key(key) {
            Err(error) => (
                None,
                Some(format!("Could not create AcoustID client: {error}")),
            ),
            Ok(client) => match client
                .lookup_by_fingerprint(
                    duration.round() as u32,
                    &fingerprint,
                    &[Metadata::RecordingIds],
                )
                .await
            {
                Ok(response) => (best_acoustid_match(response.results), None),
                Err(error) => (None, Some(error.to_string())),
            },
        },
    };

    Ok(AcoustidLookup {
        fingerprint,
        best_match,
        lookup_error,
    })
}

#[cfg(feature = "ssr")]
fn best_acoustid_match(results: Vec<acoustid::LookupResult>) -> Option<AcoustidMatch> {
    results
        .into_iter()
        .max_by(|left, right| {
            left.score
                .unwrap_or(0.0)
                .total_cmp(&right.score.unwrap_or(0.0))
        })
        .map(|result| AcoustidMatch {
            score: result.score,
            acoustid_id: result.id,
            recording_id: result
                .recordings
                .into_iter()
                .find_map(|recording| recording.id),
        })
}

#[cfg(all(test, feature = "ssr"))]
mod acoustid_match_tests {
    use super::best_acoustid_match;

    fn map_response(json: &str) -> Option<super::AcoustidMatch> {
        let response: acoustid::ApiResponse = serde_json::from_str(json).unwrap();
        best_acoustid_match(response.results)
    }

    #[test]
    fn chooses_highest_score_and_keeps_first_recording_id() {
        let result = map_response(
            r#"{
            "status":"ok",
            "results":[
                {"id":"lower","score":0.4,"recordings":[{"id":"recording-lower"}]},
                {"id":"best","score":0.93,"recordings":[{"title":"No ID"},{"id":"recording-best"}]}
            ]
        }"#,
        )
        .unwrap();

        assert_eq!(result.acoustid_id.as_deref(), Some("best"));
        assert_eq!(result.recording_id.as_deref(), Some("recording-best"));
        assert_eq!(result.score, Some(0.93));
    }

    #[test]
    fn tolerates_no_results_and_missing_recording_ids() {
        assert!(map_response(r#"{"status":"ok","results":[]}"#).is_none());

        let result = map_response(
            r#"{
            "status":"ok",
            "results":[{"id":"track","score":0.8,"recordings":[{"title":"No ID"}]}]
        }"#,
        )
        .unwrap();
        assert_eq!(result.acoustid_id.as_deref(), Some("track"));
        assert_eq!(result.recording_id, None);
    }
}

#[cfg(all(test, feature = "ssr"))]
mod fingerprint_tests {
    use super::{
        collect_regular_files, is_audio_file, parse_fpcalc_output, partition_existing_paths,
    };
    use std::{collections::HashSet, fs, path::Path, time::SystemTime};

    #[test]
    fn parses_fpcalc_output_and_rejects_invalid_durations() {
        let (fingerprint, duration) =
            parse_fpcalc_output(br#"{"duration":12.6,"fingerprint":"abc123"}"#).unwrap();
        assert_eq!(fingerprint, "abc123");
        assert_eq!(duration, 12.6);

        assert!(parse_fpcalc_output(br#"{"duration":0,"fingerprint":"x"}"#).is_err());
        assert!(parse_fpcalc_output(b"not json").is_err());
    }

    #[test]
    fn recursively_discovers_files_and_skips_paths_with_fingerprints() {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("impresario-fingerprint-{unique}"));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let existing = root.join("already-stored.mp3");
        let new_file = nested.join("new-file.flac");
        let non_audio = nested.join("cover.jpg");
        fs::write(&existing, b"audio placeholder").unwrap();
        fs::write(&new_file, b"audio placeholder").unwrap();
        fs::write(&non_audio, b"image placeholder").unwrap();

        let mut discovered = Vec::new();
        collect_regular_files(&root, &mut discovered).unwrap();
        let stored = HashSet::from([existing.to_str().unwrap().to_string()]);
        let (candidates, skipped, invalid) = partition_existing_paths(discovered, &stored);

        assert_eq!(skipped, 1);
        assert_eq!(invalid.len(), 0);
        assert_eq!(candidates, vec![new_file.to_str().unwrap().to_string()]);
        assert!(!is_audio_file(&non_audio));
        assert!(is_audio_file(Path::new("track.MP3")));

        fs::remove_dir_all(root).unwrap();
    }
}
