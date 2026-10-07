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
                                                    />
                                                })
                                            } else {
                                                Either::Right(view! {
                                                    <tr>
                                                        <td><div style="margin-left: 20px;">{item_name}</div></td>
                                                        <td>"File"</td>
                                                    </tr>
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
fn DirectoryItem(name: String, path: String, depth: usize) -> impl IntoView {
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
            <Subdirectory path=&path_clone depth=depth + 1 />
        </Show>
    }
}

#[component]
fn Subdirectory<'a>(path: &'a str, depth: usize) -> impl IntoView {
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
                                    />
                                })
                            } else {
                                Either::Right(view! {
                                    <tr>
                                        <td><div style={format!("margin-left: {}px;", depth * 20)}>{item_name}</div></td>
                                        <td>"File"</td>
                                    </tr>
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
