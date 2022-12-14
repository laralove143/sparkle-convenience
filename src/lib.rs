#![warn(
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    warnings,
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_macro_rules,
    unused_qualifications,
    variant_size_differences,
    // Nightly lints:
    // fuzzy_provenance_casts,
    // lossy_provenance_casts,
    // must_not_suspend,
    // non_exhaustive_omitted_patterns,
)]

//! A wrapper over [Twilight](https://github.com/twilight-rs/twilight) that's designed to be
//! convenient to use, without relying on callbacks and mostly following
//! Twilight patterns while making your life easier
//!
//! # Concise Startup
//!
//! ```no_run
//! # use twilight_gateway::EventTypeFlags;
//! # use twilight_model::gateway::Intents;
//! # use anyhow::Result;
//! use sparkle_convenience::Bot;
//! # async fn new() -> Result<()> {
//! Bot::new(
//!     "forgot to leak my token".to_owned(),
//!     Intents::GUILD_MESSAGES,
//!     EventTypeFlags::INTERACTION_CREATE,
//! )
//! .await?;
//! # Ok(())
//! # }
//! ```
//!
//! Yes that's really it... [`Bot`] has all the things you'd need from Twilight
//!
//! # Interaction Handling
//!
//! ```rust
//! # use twilight_model::application::interaction::Interaction;
//! # use anyhow::Result;
//! # use twilight_model::guild::Permissions;
//! use sparkle_convenience::{
//!     error::conversion::IntoError,
//!     interaction::extract::{InteractionDataExt, InteractionExt},
//!     reply::Reply,
//!     Bot,
//! };
//!
//! # async fn handle_interaction(bot: &Bot, interaction: Interaction) -> Result<()> {
//! let handle = bot.interaction_handle(&interaction);
//! match interaction.name().ok()? {
//!     "pay_respects" => {
//!         handle.defer(true).await?;
//!         // More on error handling below
//!         handle.check_permissions(Permissions::MANAGE_GUILD)?;
//!         // Say this is a user command
//!         let _very_respected_user = interaction.data.ok()?.command().ok()?.target_id.ok()?;
//!         // There are similar methods for autocomplete and modal responses
//!         handle
//!             .followup(
//!                 Reply::new()
//!                     .ephemeral()
//!                     .content("You have -1 respect now".to_owned()),
//!             )
//!             .await?;
//!     }
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! ## User-Facing Errors
//!
//! ```rust
//! # use std::fmt::{Display, Formatter};
//! # use anyhow::Result;
//! # use twilight_http::{request::channel::reaction::RequestReactionType, Client};
//! # use twilight_model::{
//! #     channel::Message,
//! #     guild::Permissions,
//! #     id::{
//! #         marker::{ChannelMarker, MessageMarker},
//! #         Id,
//! #     },
//! # };
//! use sparkle_convenience::{
//!     error::{conversion::IntoError, ErrorExt, UserError},
//!     http::message::CreateMessageExt,
//!     prettify::Prettify,
//!     reply::Reply,
//! };
//! # #[derive(Debug)]
//! # enum CustomError {};
//! #
//! # impl Display for CustomError {
//! #    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//! #        f.write_str("")
//! #    }
//! # }
//!
//! async fn wave(
//!     client: &Client,
//!     channel_id: Id<ChannelMarker>,
//!     message_id: Id<MessageMarker>,
//! ) -> Result<()> {
//!     client
//!         .create_reaction(
//!             channel_id,
//!             message_id,
//!             &RequestReactionType::Unicode { name: "????" },
//!         )
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handle_message(client: &Client, message: Message) -> Result<()> {
//!     if let Err(mut err) = wave(client, message.channel_id, message.id).await {
//!         // For example if the message was already deleted (probably by some moderation bot..)
//!         if err.ignore() {
//!             return Ok(());
//!         }
//!
//!         // Not needed in interactions thanks to `InteractionHandle::check_permissions`
//!         err.with_permissions(Permissions::READ_MESSAGE_HISTORY | Permissions::ADD_REACTIONS);
//!
//!         client
//!             .create_message(message.channel_id)
//!             .with_reply(&err_reply(&err))?
//!             .execute_ignore_permissions()
//!             .await?;
//!
//!         // `CustomError` is for your own errors
//!         if let Some(err) = err.internal::<CustomError>() {
//!             return Err(err);
//!         }
//!     }
//!
//!     Ok(())
//! }
//!
//! // Returns a reply that you can conveniently use in messages, interactions, even webhooks
//! fn err_reply(err: &anyhow::Error) -> Reply {
//!     let message = if let Some(UserError::MissingPermissions(permissions)) = err.user() {
//!         format!(
//!             "Give me those sweet permissions:\n{}",
//!             permissions.unwrap().prettify() // Also provided by this crate
//!         )
//!     } else {
//!         "Uh oh...".to_owned()
//!     };
//!     Reply::new().ephemeral().content(message)
//! }
//! ```
//!
//! ## Internal Errors
//!
//! ```no_run
//! # use anyhow::Result;
//! # use twilight_gateway::EventTypeFlags;
//! # use twilight_model::{
//! #     gateway::{event::Event, Intents},
//! #     id::Id,
//! # };
//! use sparkle_convenience::Bot;
//!
//! # async fn handle_event() -> Result<()> { Ok(()) }
//! # async fn log_example(mut bot: Bot) -> Result<()> {
//! bot.set_logging_channel(Id::new(123)).await?;
//! bot.set_logging_file("log.txt".to_owned());
//! if let Err(err) = handle_event().await {
//!     // Executes a webhook in the channel
//!     // (error message is in an attachment so don't worry if it's too long)
//!     // And appends the error to the file
//!     bot.log(format!("{err:?}")).await;
//! };
//! # Ok(())
//! # }
//! ```
//!
//! # DMs
//!
//! ```rust
//! # use anyhow::Result;
//! # use twilight_http::Client;
//! # use twilight_model::id::{marker::UserMarker, Id};
//! use sparkle_convenience::http::HttpExt;
//!
//! # async fn annoy_user(client: &Client, user_id: Id<UserMarker>) -> Result<()> {
//! client
//!     .dm_user(user_id)
//!     .await?
//!     .content("This bot is brought to you by Skillshare")?
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::{fmt::Debug, sync::Arc};

#[cfg(test)]
use futures as _;
use twilight_gateway::{cluster::Events, Cluster, EventTypeFlags, Intents};
use twilight_http::Client;
use twilight_model::{
    id::{marker::WebhookMarker, Id},
    oauth::Application,
    user::CurrentUser,
};

/// Convenient error handling
pub mod error;
/// Making HTTP requests conveniently
pub mod http;
/// Convenient interaction handling
pub mod interaction;
/// Formatting types into user-readable pretty strings
pub mod prettify;
/// The [`reply::Reply`] struct
pub mod reply;

/// All data required to make a bot run
#[derive(Debug)]
#[must_use]
pub struct Bot {
    /// Twilight's HTTP client
    pub http: Client,
    /// Twilight's gateway cluster
    pub cluster: Arc<Cluster>,
    /// The application info of the bot
    pub application: Application,
    /// The user info of the bot
    pub user: CurrentUser,
    /// The webhook to log errors using
    pub logging_webhook: Option<(Id<WebhookMarker>, String)>,
    /// The file to append errors to
    pub logging_file_path: Option<String>,
}

impl Bot {
    /// Create a new bot with the given token, intents and event types
    ///
    /// It's recommended to pass [`EventTypeFlags::all`] if using a cache
    ///
    /// By default [`Self::log`] only prints the message, see
    /// [`Self::set_logging_channel`] and [`Self::set_logging_file`]
    ///
    /// # Errors
    ///
    /// Returns [`twilight_gateway::cluster::ClusterStartError`] if creating the
    /// cluster fails
    ///
    /// Returns [`twilight_http::error::Error`] or
    /// [`twilight_http::response::DeserializeBodyError`] if getting the
    /// application info fails
    ///
    /// # Panics
    ///
    /// If not run in a Tokio runtime (under `#[tokio::main]`)
    pub async fn new(
        token: String,
        intents: Intents,
        event_types: EventTypeFlags,
    ) -> Result<(Self, Events), anyhow::Error> {
        let (cluster, events) = Cluster::builder(token.clone(), intents)
            .event_types(event_types)
            .build()
            .await?;
        let cluster_arc = Arc::new(cluster);
        let cluster_spawn = Arc::clone(&cluster_arc);
        tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        let http = Client::new(token.clone());
        let application = http.current_user_application().await?.model().await?;
        let user = http.current_user().await?.model().await?;

        Ok((
            Self {
                http,
                cluster: cluster_arc,
                application,
                user,
                logging_webhook: None,
                logging_file_path: None,
            },
            events,
        ))
    }
}
