//! Functions to deal with Stackable repositories

use kube::api::{ListParams, ObjectList};
use kube::{Api, Client};
use log::{debug, info, warn};
use std::convert::TryFrom;

use crate::provider::error::StackableError;
use package::Package;
use repository_spec::Repository;
use stackablerepository::StackableRepoProvider;

pub mod package;
pub mod repository_spec;
pub mod stackablerepository;

/// Searches for the given package in all registered repositories.
///
/// The available repositories are retrieved from the API server and if
/// the given package is provided by one of them then
/// `Ok(Some(repository))` else `Ok(None)` is returned.
///
/// If the repositories cannot be retrieved then `Err(error)` is
/// returned.
///
/// The repositories are sorted by their name to provide a deterministic
/// behavior especially for tests.
pub async fn find_repository(
    client: Client,
    package: &Package,
) -> Result<Option<StackableRepoProvider>, StackableError> {
    let repositories = retrieve_repositories(client).await?;

    let mut repository_providers = repositories
        .iter()
        .filter_map(convert_to_repo_provider)
        .collect::<Vec<_>>();

    repository_providers.sort_unstable_by_key(|x| x.name.to_owned());

    let maybe_repository = choose_repository_provider(&mut repository_providers, package).await;

    if let Some(repository) = &maybe_repository {
        debug!(
            "Package [{}] found in repository [{}]",
            &package, &repository
        );
    } else {
        info!(
            "Package [{}] not found in the following repositories: [{:?}]",
            &package, &repository_providers
        );
    }

    Ok(maybe_repository)
}

/// Retrieves all Stackable repositories in the default namespace from
/// the API server.
async fn retrieve_repositories(client: Client) -> Result<ObjectList<Repository>, StackableError> {
    let api: Api<Repository> = Api::namespaced(client, "default");
    let repositories = api.list(&ListParams::default()).await?;
    Ok(repositories)
}

/// Converts the given Stackable repository into a repository provider.
///
/// If this fails then a warning is emitted and `None` is returned.
///
/// The returned repository provider does not yet contain the list of
/// provided packages.
fn convert_to_repo_provider(repository: &Repository) -> Option<StackableRepoProvider> {
    let result = StackableRepoProvider::try_from(repository);

    if let Err(error) = &result {
        warn!("Invalid repository definition: {}", error);
    }

    result.ok()
}

/// Retrieves the provided packages for the given repository providers
/// and returns the first provider which provides the given package or
/// `None` if none provides it.
async fn choose_repository_provider(
    repo_providers: &mut [StackableRepoProvider],
    package: &Package,
) -> Option<StackableRepoProvider> {
    for repo_provider in repo_providers {
        if let Ok(true) = repo_provider.provides_package(package.to_owned()).await {
            return Some(repo_provider.to_owned());
        }
    }
    None
}
