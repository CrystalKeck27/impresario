use leptos::{either::Either, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};

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
    recording_matches: Vec<RecordingCandidate>,
    lookup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcoustidMatch {
    score: Option<f64>,
    acoustid_id: Option<String>,
    recording_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RecordingCandidate {
    recording_id: String,
    score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum IdentifyStatus {
    Unprocessed,
    NotFound,
    Matched,
    Multiple,
    Conflict,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct IdentifyFile {
    relative_path: String,
    status: IdentifyStatus,
    matches: Vec<RecordingCandidate>,
    selected_recording_id: Option<String>,
    message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct IdentifySaveResult {
    conflict: bool,
}

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
                    <Route path=StaticSegment("identify") view=IdentifyPage/>
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
        <a href="/identify"><button>"Identify music"</button></a>
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
fn IdentifyPage() -> impl IntoView {
    let source_path = RwSignal::new(".".to_string());
    let destination_path = RwSignal::new("../identified".to_string());
    let files = RwSignal::new(Vec::<IdentifyFile>::new());
    let selections = RwSignal::new(std::collections::HashMap::<String, String>::new());
    let running = RwSignal::new(false);
    let scanned = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let initial_files = Resource::new(
        move || source_path.get(),
        |source| async move { list_identify_files(source).await.unwrap_or_default() },
    );

    let run_scan = move |_| {
        running.set(true);
        scanned.set(false);
        feedback.set(None);
        files.set(Vec::new());
        selections.set(std::collections::HashMap::new());
        let source = source_path.get_untracked();
        let destination = destination_path.get_untracked();
        leptos::task::spawn_local(async move {
            match identify_directory(source, destination).await {
                Ok(found) => files.set(found),
                Err(error) => feedback.set(Some(error.to_string())),
            }
            scanned.set(true);
            running.set(false);
        });
    };

    view! {
        <Title text="Identify music · Impresario" />
        <section class="identify-page">
            <header class="identify-page__header">
                <p class="identify-page__eyebrow">"AcoustID · MusicBrainz"</p>
                <h1>"Identify music"</h1>
                <p>"Fingerprint audio files and save the matching MusicBrainz recording ID beside each one in the destination tree."</p>
            </header>

            <div class="identify-controls">
                <label class="identify-controls__field">
                    <span>"Source directory"</span>
                    <input
                        type="text"
                        aria-label="Source directory"
                        disabled=move || running.get()
                        prop:value=move || source_path.get()
                        placeholder="/path/to/music"
                        on:input=move |ev| {
                            source_path.set(event_target_value(&ev));
                            scanned.set(false);
                            files.set(Vec::new());
                        }
                    />
                </label>
                <label class="identify-controls__field">
                    <span>"Destination directory"</span>
                    <input
                        type="text"
                        aria-label="Destination directory"
                        disabled=move || running.get()
                        prop:value=move || destination_path.get()
                        placeholder="/path/to/identified"
                        on:input=move |ev| {
                            destination_path.set(event_target_value(&ev));
                            scanned.set(false);
                            files.set(Vec::new());
                        }
                    />
                </label>
                <button class="identify-controls__run" type="button" on:click=run_scan disabled=move || running.get()>
                    {move || if running.get() { "Scanning…" } else { "Run identification" }}
                </button>
            </div>

            <p class="identify-page__note">"Scans supported audio files recursively. Output folders mirror the source tree; a sidecar is named like track.flac.recording.mbid. Destination cannot be inside the source."</p>

            {move || feedback.get().map(|message| view! {
                <p class="identify-feedback identify-feedback--error" role="alert">{message}</p>
            })}

            <section class="identify-explorer" aria-label="Music identification results">
                <div class="identify-explorer__heading">
                    <div>
                        <p class="identify-page__eyebrow">"SOURCE FILES"</p>
                        <h2>{move || source_path.get()}</h2>
                    </div>
                    <div class="identify-legend" aria-label="Match status legend">
                        <span><i class="identify-legend__dot identify-legend__dot--default"></i>"Not scanned"</span>
                        <span><i class="identify-legend__dot identify-legend__dot--not-found"></i>"Not found"</span>
                        <span><i class="identify-legend__dot identify-legend__dot--matched"></i>"Matched"</span>
                        <span><i class="identify-legend__dot identify-legend__dot--multiple"></i>"Choose match"</span>
                    </div>
                </div>
                <div class="identify-file-list">
                    <Suspense fallback=|| view! { <p class="identify-empty">"Loading source files…"</p> }>
                    {move || {
                        let found = if scanned.get() {
                            files.get()
                        } else {
                            initial_files.get().unwrap_or_default()
                        };
                        if found.is_empty() {
                            return vec![view! {
                                <p class="identify-empty">{move || if running.get() {
                                    "Fingerprinting source files…"
                                } else {
                                    if scanned.get() { "No supported audio files found in this source directory." } else { "Choose source and destination directories, then run identification." }
                                }}</p>
                            }.into_any()];
                        }

                        found.into_iter().map(|file| {
                            let relative_path = file.relative_path.clone();
                            let select_key = relative_path.clone();
                            let candidate_options = file.matches.iter().map(|candidate| {
                                let id = candidate.recording_id.clone();
                                let score = candidate.score.map(|score| format!(" · {:.0}%", score * 100.0)).unwrap_or_default();
                                view! { <option value=id.clone()>{format!("{}{}", id, score)}</option> }
                            }).collect::<Vec<_>>();
                            let candidate_links = file.matches.iter().map(|candidate| {
                                let id = candidate.recording_id.clone();
                                let score = candidate.score
                                    .map(|score| format!("{:.0}%", score * 100.0))
                                    .unwrap_or_else(|| "score unavailable".to_string());
                                view! {
                                    <a href=format!("https://musicbrainz.org/recording/{id}") target="_blank" rel="noopener noreferrer">{format!("{} · {}", candidate.recording_id, score)}</a>
                                }
                            }).collect::<Vec<_>>();
                            let row_class = match file.status {
                                IdentifyStatus::Unprocessed => "identify-file identify-file--default",
                                IdentifyStatus::NotFound => "identify-file identify-file--not-found",
                                IdentifyStatus::Matched => "identify-file identify-file--matched",
                                IdentifyStatus::Multiple => "identify-file identify-file--multiple",
                                IdentifyStatus::Conflict => "identify-file identify-file--conflict",
                                IdentifyStatus::Error => "identify-file identify-file--error",
                            };
                            let status_text = match file.status {
                                IdentifyStatus::Unprocessed => "Not scanned",
                                IdentifyStatus::NotFound => "No MusicBrainz recording found",
                                IdentifyStatus::Matched => "Matched",
                                IdentifyStatus::Multiple => "Select the best match",
                                IdentifyStatus::Conflict => "File conflict: sidecar contents differ",
                                IdentifyStatus::Error => "Could not identify this file",
                            };
                            let chosen_id = file.selected_recording_id.clone().or_else(|| {
                                (file.matches.len() == 1).then(|| file.matches[0].recording_id.clone())
                            }).unwrap_or_default();
                            let link_id = chosen_id.clone();
                            let matched_link = (!chosen_id.is_empty() && file.status != IdentifyStatus::Multiple).then(|| view! {
                                <a href=format!("https://musicbrainz.org/recording/{link_id}") target="_blank" rel="noopener noreferrer">"Open recording on MusicBrainz"</a>
                            });
                            let multiple_controls = (file.status == IdentifyStatus::Multiple).then(|| view! {
                                <div class="identify-file__actions">
                                    <label>
                                        <span class="identify-sr-only">"Choose a recording for " {relative_path.clone()}</span>
                                        <select
                                            aria-label=format!("Recording match for {}", relative_path)
                                            on:change=move |ev| {
                                                selections.update(|values| { values.insert(select_key.clone(), event_target_value(&ev)); });
                                            }
                                        >
                                            <option value="">"Choose a recording…"</option>
                                            {candidate_options}
                                        </select>
                                    </label>
                                    <div class="identify-file__candidate-links">{candidate_links}</div>
                                    <button type="button" on:click={
                                        let path = relative_path.clone();
                                        move |_| {
                                            let id = selections.get().get(&path).cloned().unwrap_or_default();
                                            if !id.is_empty() {
                                                save_identification_from_ui(source_path, destination_path, files, feedback, path.clone(), id, false);
                                            }
                                        }
                                    } disabled={
                                        let path = relative_path.clone();
                                        move || selections.get().get(&path).map(String::is_empty).unwrap_or(true)
                                    }>{"Save selected match"}</button>
                                </div>
                            });
                            let conflict_control = (file.status == IdentifyStatus::Conflict).then(|| view! {
                                <button type="button" class="identify-file__overwrite" on:click={
                                    let path = relative_path.clone();
                                    let id = chosen_id.clone();
                                    move |_| save_identification_from_ui(source_path, destination_path, files, feedback, path.clone(), id.clone(), true)
                                }>{"Replace sidecar with this match"}</button>
                            });
                            view! {
                                <article class=row_class>
                                    <div class="identify-file__main">
                                        <span class="identify-file__icon" aria-hidden="true">"♫"</span>
                                        <div class="identify-file__details">
                                            <strong>{relative_path}</strong>
                                            <span class="identify-file__status">{status_text}</span>
                                            {file.message.map(|message| view! { <span class="identify-file__message">{message}</span> })}
                                            {matched_link}
                                            {multiple_controls}
                                            {conflict_control}
                                        </div>
                                    </div>
                                </article>
                            }.into_any()
                        }).collect::<Vec<_>>()
                    }}
                    </Suspense>
                </div>
            </section>
        </section>
    }
}

fn save_identification_from_ui(
    source_path: RwSignal<String>,
    destination_path: RwSignal<String>,
    files: RwSignal<Vec<IdentifyFile>>,
    feedback: RwSignal<Option<String>>,
    relative_path: String,
    recording_id: String,
    overwrite: bool,
) {
    let source = source_path.get_untracked();
    let destination = destination_path.get_untracked();
    leptos::task::spawn_local(async move {
        match save_recording_id(
            source,
            destination,
            relative_path.clone(),
            recording_id.clone(),
            overwrite,
        )
        .await
        {
            Ok(result) => files.update(|entries| {
                if let Some(file) = entries
                    .iter_mut()
                    .find(|file| file.relative_path == relative_path)
                {
                    file.status = if result.conflict {
                        IdentifyStatus::Conflict
                    } else {
                        IdentifyStatus::Matched
                    };
                    file.selected_recording_id = Some(recording_id.clone());
                    file.message = if result.conflict {
                        Some(
                            "A different sidecar already exists; confirm replacement to continue."
                                .to_string(),
                        )
                    } else {
                        None
                    };
                }
            }),
            Err(error) => feedback.set(Some(error.to_string())),
        }
    });
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
    acoustid_lookup_for_path(path)
        .await
        .map_err(ServerFnError::ServerError)
}

#[cfg(feature = "ssr")]
async fn acoustid_lookup_for_path(path: String) -> Result<AcoustidLookup, String> {
    use acoustid::{AcoustIDClient, Metadata};
    use std::{fs, process::Stdio};
    use tokio::process::Command;

    #[derive(Deserialize)]
    struct FpcalcOutput {
        duration: f64,
        fingerprint: String,
    }

    let path = fs::canonicalize(&path).map_err(|e| format!("Could not access file: {e}"))?;
    let metadata = fs::metadata(&path).map_err(|e| format!("Could not access file: {e}"))?;
    if !metadata.is_file() {
        return Err("Fingerprint input must be a file".to_string());
    }

    let output = Command::new("fpcalc")
        .arg("-json")
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Could not run fpcalc: {e}"))?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            format!("fpcalc exited with status {}", output.status)
        } else {
            format!("fpcalc failed: {message}")
        });
    }

    let fpcalc: FpcalcOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Could not parse fpcalc output: {e}"))?;
    if !fpcalc.duration.is_finite() || fpcalc.duration <= 0.0 || fpcalc.duration > u32::MAX as f64 {
        return Err("fpcalc returned an invalid duration".to_string());
    }

    let fingerprint = fpcalc.fingerprint;
    let application_key = std::env::var("ACOUSTID_APPLICATION_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty());
    let (best_match, recording_matches, lookup_error) = match application_key {
        None => (
            None,
            Vec::new(),
            Some("ACOUSTID_APPLICATION_KEY is not set on the server".to_string()),
        ),
        Some(key) => match AcoustIDClient::with_application_key(key) {
            Err(error) => (
                None,
                Vec::new(),
                Some(format!("Could not create AcoustID client: {error}")),
            ),
            Ok(client) => match client
                .lookup_by_fingerprint(
                    fpcalc.duration.round() as u32,
                    &fingerprint,
                    &[Metadata::RecordingIds],
                )
                .await
            {
                Ok(response) => {
                    let matches = recording_candidates(&response.results);
                    (best_acoustid_match(response.results), matches, None)
                }
                Err(error) => (None, Vec::new(), Some(error.to_string())),
            },
        },
    };

    Ok(AcoustidLookup {
        fingerprint,
        best_match,
        recording_matches,
        lookup_error,
    })
}

#[cfg(feature = "ssr")]
fn recording_candidates(results: &[acoustid::LookupResult]) -> Vec<RecordingCandidate> {
    let mut candidates: Vec<RecordingCandidate> = Vec::new();
    for result in results {
        for recording_id in result.recordings.iter().filter_map(|recording| recording.id.as_ref()) {
            if let Some(candidate) = candidates.iter_mut().find(|candidate| &candidate.recording_id == recording_id) {
                if result.score > candidate.score {
                    candidate.score = result.score;
                }
            } else {
                candidates.push(RecordingCandidate {
                    recording_id: recording_id.clone(),
                    score: result.score,
                });
            }
        }
    }
    candidates.sort_by(|left, right| {
        right.score.unwrap_or(0.0).total_cmp(&left.score.unwrap_or(0.0))
    });
    candidates
}

#[server]
async fn identify_directory(
    source: String,
    destination: String,
) -> Result<Vec<IdentifyFile>, ServerFnError<String>> {
    use std::{fs, path::Path};

    let (source_root, destination_root) = prepare_identify_roots(&source, &destination)
        .map_err(ServerFnError::ServerError)?;
    let mut source_files = Vec::new();
    collect_audio_files(&source_root, &source_root, &mut source_files)
        .map_err(ServerFnError::ServerError)?;
    source_files.sort();

    let mut results = Vec::with_capacity(source_files.len());
    for path in source_files {
        let relative_path = path
            .strip_prefix(&source_root)
            .map_err(|error| ServerFnError::ServerError(error.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        let lookup = match acoustid_lookup_for_path(path.to_string_lossy().into_owned()).await {
            Ok(lookup) => lookup,
            Err(error) => {
                results.push(IdentifyFile {
                    relative_path,
                    status: IdentifyStatus::Error,
                    matches: Vec::new(),
                    selected_recording_id: None,
                    message: Some(error),
                });
                continue;
            }
        };
        if let Some(error) = lookup.lookup_error {
            results.push(IdentifyFile {
                relative_path,
                status: IdentifyStatus::Error,
                matches: Vec::new(),
                selected_recording_id: None,
                message: Some(error),
            });
            continue;
        }

        let matches = lookup.recording_matches;
        let (status, message) = match matches.len() {
            0 => (IdentifyStatus::NotFound, None),
            1 => {
                match write_recording_id(&source_root, &destination_root, &relative_path, &matches[0].recording_id, false) {
                    Ok(true) => (IdentifyStatus::Conflict, Some("A different sidecar already exists; confirm replacement to continue.".to_string())),
                    Ok(false) => (IdentifyStatus::Matched, None),
                    Err(error) => (IdentifyStatus::Error, Some(error)),
                }
            }
            _ => (IdentifyStatus::Multiple, None),
        };
        let selected_recording_id = (matches.len() == 1).then(|| matches[0].recording_id.clone());
        results.push(IdentifyFile { relative_path, status, matches, selected_recording_id, message });
    }

    // Ensure destination creation has not redirected output through a symlink.
    let final_destination = fs::canonicalize(Path::new(&destination)).map_err(|error| {
        ServerFnError::ServerError(format!("Could not resolve destination directory: {error}"))
    })?;
    if final_destination != destination_root {
        return Err(ServerFnError::ServerError("Destination path changed during scan".to_string()));
    }
    Ok(results)
}

#[server]
async fn list_identify_files(source: String) -> Result<Vec<IdentifyFile>, ServerFnError<String>> {
    use std::{fs, path::Path};

    let source_root = fs::canonicalize(&source)
        .map_err(|error| ServerFnError::ServerError(format!("Could not access source directory: {error}")))?;
    if !source_root.is_dir() {
        return Err(ServerFnError::ServerError("Source path must be a directory".to_string()));
    }
    let mut source_files = Vec::new();
    collect_audio_files(&source_root, Path::new(&source_root), &mut source_files)
        .map_err(ServerFnError::ServerError)?;
    source_files.sort();
    Ok(source_files.into_iter().filter_map(|path| {
        path.strip_prefix(&source_root).ok().map(|relative| IdentifyFile {
            relative_path: relative.to_string_lossy().replace('\\', "/"),
            status: IdentifyStatus::Unprocessed,
            matches: Vec::new(),
            selected_recording_id: None,
            message: None,
        })
    }).collect())
}

#[server]
async fn save_recording_id(
    source: String,
    destination: String,
    relative_path: String,
    recording_id: String,
    overwrite: bool,
) -> Result<IdentifySaveResult, ServerFnError<String>> {
    if recording_id.is_empty()
        || recording_id.len() > 64
        || !recording_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(ServerFnError::ServerError("Invalid MusicBrainz recording ID".to_string()));
    }
    let (source_root, destination_root) = prepare_identify_roots(&source, &destination)
        .map_err(ServerFnError::ServerError)?;
    let conflict = write_recording_id(
        &source_root,
        &destination_root,
        &relative_path,
        &recording_id,
        overwrite,
    )
    .map_err(ServerFnError::ServerError)?;
    Ok(IdentifySaveResult { conflict })
}

#[cfg(feature = "ssr")]
fn prepare_identify_roots(
    source: &str,
    destination: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    use std::{fs, path::PathBuf};

    let source_root = fs::canonicalize(source)
        .map_err(|error| format!("Could not access source directory: {error}"))?;
    if !source_root.is_dir() {
        return Err("Source path must be a directory".to_string());
    }

    let mut nearest_existing = PathBuf::from(destination);
    let mut missing_components = Vec::new();
    while !nearest_existing.exists() {
        let component = nearest_existing
            .file_name()
            .ok_or_else(|| "Could not resolve destination directory".to_string())?
            .to_os_string();
        missing_components.push(component);
        if !nearest_existing.pop() {
            return Err("Could not resolve destination directory".to_string());
        }
    }
    let mut destination_candidate = fs::canonicalize(nearest_existing)
        .map_err(|error| format!("Could not access destination directory: {error}"))?;
    for component in missing_components.iter().rev() {
        destination_candidate.push(component);
    }
    if destination_candidate.starts_with(&source_root) {
        return Err("Destination directory cannot be inside the source directory".to_string());
    }
    fs::create_dir_all(&destination_candidate)
        .map_err(|error| format!("Could not create destination directory: {error}"))?;
    let destination_root = fs::canonicalize(destination)
        .map_err(|error| format!("Could not access destination directory: {error}"))?;
    if destination_root.starts_with(&source_root) {
        return Err("Destination directory cannot be inside the source directory".to_string());
    }
    Ok((source_root, destination_root))
}

#[cfg(feature = "ssr")]
fn collect_audio_files(
    source_root: &std::path::Path,
    directory: &std::path::Path,
    files: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    use std::fs;

    for entry in fs::read_dir(directory).map_err(|error| format!("Could not read {}: {error}", directory.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            let canonical = fs::canonicalize(&path).map_err(|error| error.to_string())?;
            if canonical.starts_with(source_root) {
                collect_audio_files(source_root, &canonical, files)?;
            }
        } else if metadata.is_file() && is_supported_audio(&path) {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn is_supported_audio(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "mp3" | "flac" | "ogg" | "opus" | "m4a" | "mp4" | "aac" | "wav" | "aiff" | "aif" | "wma" | "ape" | "wv"
            )
        })
        .unwrap_or(false)
}

#[cfg(feature = "ssr")]
fn write_recording_id(
    source_root: &std::path::Path,
    destination_root: &std::path::Path,
    relative_path: &str,
    recording_id: &str,
    overwrite: bool,
) -> Result<bool, String> {
    use std::{fs, path::{Component, Path, PathBuf}};

    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("Invalid source-relative file path".to_string());
    }
    let source_file = fs::canonicalize(source_root.join(relative))
        .map_err(|error| format!("Could not access source file: {error}"))?;
    if !source_file.starts_with(source_root) || !source_file.is_file() {
        return Err("Source file is outside the selected directory".to_string());
    }

    let mut sidecar_relative = PathBuf::new();
    for component in relative.parent().into_iter().flat_map(Path::components) {
        sidecar_relative.push(component.as_os_str());
    }
    let filename = relative.file_name().ok_or_else(|| "Invalid source filename".to_string())?;
    let mut sidecar_name = filename.to_os_string();
    sidecar_name.push(".recording.mbid");
    sidecar_relative.push(sidecar_name);
    let sidecar = destination_root.join(sidecar_relative);
    let parent = sidecar.parent().ok_or_else(|| "Invalid destination path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("Could not create output directory: {error}"))?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| error.to_string())?;
    if !canonical_parent.starts_with(destination_root) {
        return Err("Output path escapes the destination directory".to_string());
    }
    if let Ok(metadata) = fs::symlink_metadata(&sidecar) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("Output sidecar is not a regular file".to_string());
        }
        let current = fs::read_to_string(&sidecar)
            .map_err(|error| format!("Could not read existing sidecar: {error}"))?;
        if current == recording_id {
            return Ok(false);
        }
        if !overwrite {
            return Ok(true);
        }
    }
    fs::write(sidecar, recording_id)
        .map_err(|error| format!("Could not write recording ID: {error}"))?;
    Ok(false)
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
    use super::{best_acoustid_match, prepare_identify_roots, recording_candidates, write_recording_id};
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

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

    #[test]
    fn deduplicates_recording_ids_and_keeps_the_best_score() {
        let response: acoustid::ApiResponse = serde_json::from_str(
            r#"{
              "status":"ok",
              "results":[
                {"score":0.4,"recordings":[{"id":"recording-a"},{"id":"recording-b"}]},
                {"score":0.9,"recordings":[{"id":"recording-a"}]}
              ]
            }"#,
        ).unwrap();
        let candidates = recording_candidates(&response.results);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].recording_id, "recording-a");
        assert_eq!(candidates[0].score, Some(0.9));
        assert_eq!(candidates[1].recording_id, "recording-b");
    }

    fn test_directory() -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("impresario-identify-{}-{unique}", std::process::id()))
    }

    #[test]
    fn sidecar_name_is_exact_and_conflicting_content_requires_overwrite() {
        let temp = test_directory();
        let source = temp.join("music");
        let destination = temp.join("identified");
        fs::create_dir_all(source.join("album")).unwrap();
        fs::create_dir_all(&destination).unwrap();
        fs::write(source.join("album/track.flac"), b"audio fixture").unwrap();
        let source = fs::canonicalize(source).unwrap();
        let destination = fs::canonicalize(destination).unwrap();

        assert!(!write_recording_id(&source, &destination, "album/track.flac", "recording-a", false).unwrap());
        let sidecar = destination.join("album/track.flac.recording.mbid");
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "recording-a");
        assert!(!write_recording_id(&source, &destination, "album/track.flac", "recording-a", false).unwrap());
        assert!(write_recording_id(&source, &destination, "album/track.flac", "recording-b", false).unwrap());
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "recording-a");
        assert!(!write_recording_id(&source, &destination, "album/track.flac", "recording-b", true).unwrap());
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "recording-b");

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn sidecar_writer_rejects_paths_outside_the_source_tree() {
        let temp = test_directory();
        let source = temp.join("music");
        let destination = temp.join("identified");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&destination).unwrap();
        let source = fs::canonicalize(source).unwrap();
        let destination = fs::canonicalize(destination).unwrap();

        assert!(write_recording_id(&source, &destination, "../escape.flac", "recording-a", false).is_err());
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn identify_roots_reject_nested_destination_without_creating_it() {
        let temp = test_directory();
        let source = temp.join("music");
        let nested_destination = source.join("identified/deep");
        fs::create_dir_all(&source).unwrap();
        let source = fs::canonicalize(source).unwrap();

        assert!(prepare_identify_roots(
            source.to_str().unwrap(),
            nested_destination.to_str().unwrap()
        )
        .is_err());
        assert!(!nested_destination.exists());

        fs::remove_dir_all(temp).unwrap();
    }
}
