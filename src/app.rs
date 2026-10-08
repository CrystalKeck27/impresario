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
                    <Route path=StaticSegment("base64") view=Base64Page/>
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
        <a href="/base64"><button>"i32 to Base64"</button></a>
    }
}

#[component]
fn Base64Page() -> impl IntoView {
    let input = RwSignal::new(String::new());
    let result = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match encode_i32_list(&input.get()) {
            Ok(encoded) => {
                result.set(Some(encoded));
                error.set(None);
            }
            Err(message) => {
                result.set(None);
                error.set(Some(message));
            }
        }
    };

    view! {
        <section class="base64-page">
            <header class="base64-page__header">
                <p class="tags-panel__eyebrow">"Converter"</p>
                <h1>"i32 list to Base64"</h1>
                <p>"Enter decimal 32-bit integers separated by commas. Each value is encoded as 4 little-endian bytes before Base64 encoding."</p>
            </header>
            <form class="base64-form" on:submit=submit>
                <label class="root-path-form__label" for="i32-values">"Comma-separated i32 values"</label>
                <textarea
                    id="i32-values"
                    class="base64-form__input"
                    rows="4"
                    placeholder="e.g. 1, -2, 2147483647"
                    prop:value=move || input.get()
                    on:input=move |ev| input.set(event_target_value(&ev))
                ></textarea>
                <button class="root-path-form__button" type="submit">"Convert to Base64"</button>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="base64-page__error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || result.get().is_some()>
                <section class="base64-result" aria-live="polite">
                    <h2>"Base64 result"</h2>
                    <code>{move || result.get().unwrap_or_default()}</code>
                </section>
            </Show>
        </section>
    }
}

fn encode_i32_list(input: &str) -> Result<String, String> {
    use base64::Engine as _;

    if input.trim().is_empty() {
        return Err("Enter at least one decimal i32 value.".to_string());
    }

    let mut bytes = Vec::new();
    for (index, value) in input.split(',').enumerate() {
        let value = value.trim();
        if value.is_empty() {
            return Err(format!("Value {} is empty; check the comma-separated list.", index + 1));
        }
        let parsed = value.parse::<i32>().map_err(|_| {
            format!("Value {} ({value}) is not a valid decimal i32.", index + 1)
        })?;
        bytes.extend_from_slice(&parsed.to_le_bytes());
    }

    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
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
    let tags = Resource::new(
        move || selected_file.get(),
        |selected_path| async move {
            match selected_path {
                Some(path) => fetch_file_content(path)
                    .await
                    .map(|items| {
                        items
                            .into_iter()
                            .map(|(key, value)| FileTag { key, value })
                            .collect::<Vec<_>>()
                    }),
                None => Ok(vec![]),
            }
        },
    );
    let fingerprint = Resource::new(
        move || selected_file.get(),
        |selected_path| async move {
            match selected_path {
                Some(path) => calculate_acoustid(path).await,
                None => Ok(String::new()),
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
                <Transition fallback=move || view! {
                    <p class="tags-panel__status">"Calculating fingerprint..."</p>
                }>
                    <div class="tags-panel__fingerprint-value">
                        {move || match selected_file.get() {
                            None => "Select an audio track to calculate its fingerprint.".to_string(),
                            Some(_) => match fingerprint.get() {
                                None => "Calculating fingerprint...".to_string(),
                                Some(Err(error)) => format!("Could not calculate fingerprint: {error}"),
                                Some(Ok(value)) => value,
                            },
                        }}
                    </div>
                </Transition>
            </section>

            <Transition fallback=move || view! {
                <p class="tags-panel__status">"Loading tags..."</p>
            }>
                {move || match selected_file.get() {
                    None => Either::Left(view! {
                        <p class="tags-panel__status">{"Choose a file in the explorer to inspect its tags.".to_string()}</p>
                    }),
                    Some(_) => match tags.get() {
                        None => Either::Left(view! {
                            <p class="tags-panel__status">{"Loading tags...".to_string()}</p>
                        }),
                        Some(Err(error)) => Either::Left(view! {
                            <p class="tags-panel__status">{format!("Could not read tags: {error}")}</p>
                        }),
                        Some(Ok(file_tags)) if file_tags.is_empty() => Either::Left(view! {
                            <p class="tags-panel__status">{"No tags found for this file.".to_string()}</p>
                        }),
                        Some(Ok(file_tags)) => Either::Right(view! {
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
                        }),
                    },
                }}
            </Transition>
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
pub async fn fetch_file_content(path: String) -> Result<Vec<(String, String)>, ServerFnError<String>> {
    use lofty::file::TaggedFileExt as _;

    let tags = lofty::read_from_path(path);

    let mapped = tags.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let tag = match mapped.primary_tag() {
		Some(primary_tag) => primary_tag,
		// If the "primary" tag doesn't exist, we just grab the
		// first tag we can find. Realistically, a tag reader would likely
		// iterate through the tags to find a suitable one.
		None => mapped.first_tag().ok_or(ServerFnError::ServerError("No tag found for file".to_string()))?,
	};

    Ok(tag.items().map(|item| (item.key().map_key(lofty::tag::TagType::VorbisComments).unwrap_or("N/A").to_string(), item.value().clone().into_string().unwrap_or("N/A".to_string()))).collect::<Vec<_>>())
}

#[server]
pub async fn calculate_acoustid(path: String) -> Result<String, ServerFnError<String>> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::TrackType;

    // Read the audio file and extract raw audio samples using symphonia
    let file = Box::new(std::fs::File::open(&path).map_err(|e| ServerFnError::ServerError(e.to_string()))?);

    let mss = symphonia::core::io::MediaSourceStream::new(file, Default::default());


    // Create a hint to help the format registry guess what format reader is appropriate. In this
    // example we'll leave it empty.
    let hint = Hint::new();

    // Use the default options when reading and decoding.
    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();
    let dec_opts: AudioDecoderOptions = Default::default();

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    // Get the default audio track.
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| ServerFnError::ServerError("No audio track found".to_string()))?;

    // Create a decoder for the track.
    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(|params| params.audio())
        .ok_or_else(|| ServerFnError::ServerError("No audio codec parameters found".to_string()))?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(codec_params, &dec_opts)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let track_id = track.id;

    let mut samples: Vec<i16> = Default::default();
    let mut total_sample_count = 0;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| ServerFnError::ServerError("Audio sample rate is unavailable".to_string()))?;
    let channels = codec_params
        .channels
        .as_ref()
        .ok_or_else(|| ServerFnError::ServerError("Audio channel layout is unavailable".to_string()))?
        .count() as u16;

    while let Some(packet) = format.next_packet().map_err(|e| ServerFnError::ServerError(e.to_string()))? {
        // If the packet does not belong to the selected track, skip it.
        if packet.track_id != track_id {
            continue;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                // The decoded audio samples may now be accessed via the generic audio buffer
                // returned by the decoder. You may match on the buffer to access a sample-format
                // specific buffer, or use generic routines to copy out the audio samples in the
                // desired sample format.
                //
                // In the example below, we will copy the all the samples into a vector in
                // the f32 sample format in channel interleaved order.

                let mut packet_samples = vec![i16::MIN; audio_buf.samples_interleaved()];

                // Copy the audio samples from the generic audio buffer to the vector in interleaved
                // order. The sample format to convert to is inferred from the type of the Vec.
                audio_buf.copy_to_slice_interleaved(&mut packet_samples);

                // Sum up the total number of samples.
                total_sample_count += packet_samples.len();
                print!("\rDecoded {total_sample_count} samples");
                samples.extend(packet_samples);
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }

    let fprint = chromaprint::fingerprint_audio(&samples, sample_rate, channels, chromaprint::Algorithm::default()).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    // Placeholder implementation
    Ok(fprint.encoded().to_string())
}
