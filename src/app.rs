use leptos::{either::Either, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use thaw::*;

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
    view! {
        <h1>"Files Page"</h1>
        <FileList path="."/>
    }
}

#[component]
fn FileList<'a>(path: &'a str) -> impl IntoView {
    let files = OnceResource::new(fetch_files_in_directory(path.to_string()));

    let files_suspense = move || {
        Suspend::new(async move {
            files.await.map(|file_names| {
                if file_names.is_empty() {
                    Either::Left(view! { <p>"No files found."</p> })
                } else {
                    Either::Right(
                        file_names.into_iter().map(|file_name| {
                            view! { <TreeItem item_type=TreeItemType::Branch>{file_name}</TreeItem> }
                        }).collect::<Vec<_>>()
                    )
                }
            }).map_err(|e| e.to_string())
        })
    };

    view! {
        <div>
            <Transition fallback=|| view! { <p>"Loading files..."</p> }>
                <ErrorBoundary fallback=|_error| view! { <p>"Error loading files: "</p> }>
                <Tree>
                    {files_suspense()}
                </Tree>
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

/// Fetches files in a directory and returns a list of file names. This function is only available on the server side.
#[server]
pub async fn fetch_files_in_directory(path: String) -> Result<Vec<String>, ServerFnError<String>> {
    use std::fs;

    let entries = fs::read_dir(path).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let mut file_names = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        if let Some(file_name) = entry.file_name().to_str() {
            file_names.push(file_name.to_string());
        }
    }

    Ok(file_names)
}

