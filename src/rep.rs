//! Rust representations of Github API data structures

use super::{SortDirection, Error, State as StdState};
use super::hooks::WebHookContentType;
use super::issues::Sort as IssueSort;
use super::search::SearchIssuesSort;
use super::repositories::{Sort as RepoSort, Affiliation, Type as RepoType,
                          Visibility as RepoVisibility, OrgRepoType};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::option::Option;
use url::form_urlencoded;

use super::url;
extern crate serializable_enum;
extern crate serde;
extern crate serde_json;

// this file is input for rep.rs output
use self::super::{Github, Result};

use url::Url;

#[derive(Debug, Deserialize, PartialEq)]
pub struct FieldErr {
    pub resource: String,
    pub field: Option<String>,
    pub code: String,
    pub message: Option<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ClientError {
    pub message: String,
    pub errors: Option<Vec<FieldErr>>,
}

#[derive(Debug, Deserialize)]
pub struct Deployment {
    pub url: String,
    pub id: u64,
    pub sha: String,
    #[serde(rename="ref")]
    pub commit_ref: String,
    pub task: String,
    pub payload: serde_json::Value,
    pub environment: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,
    pub creator: User,
    pub created_at: String,
    pub updated_at: String,
    pub statuses_url: String,
    pub repository_url: String,
}

#[derive(Debug, Serialize)]
pub struct DeploymentOptions {
    #[serde(rename="ref")]
    pub commit_ref: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub auto_merge: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub required_contexts: Option<Vec<String>>,
    /// contents of payload should be valid JSON
    #[serde(skip_serializing_if="Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,
}

impl DeploymentOptions {
    pub fn builder<C>(commit: C) -> DeploymentOptionsBuilder
        where C: Into<String>
    {
        DeploymentOptionsBuilder::new(commit)
    }
}

#[derive(Default)]
pub struct DeploymentOptionsBuilder {
    pub commit_ref: String,
    pub task: Option<String>,
    pub auto_merge: Option<bool>,
    pub required_contexts: Option<Vec<String>>,
    pub payload: Option<String>,
    pub environment: Option<String>,
    pub description: Option<String>,
}

impl DeploymentOptionsBuilder {
    pub fn new<C>(commit: C) -> DeploymentOptionsBuilder
        where C: Into<String>
    {
        DeploymentOptionsBuilder { commit_ref: commit.into(), ..Default::default() }
    }

    pub fn task<T>(&mut self, task: T) -> &mut DeploymentOptionsBuilder
        where T: Into<String>
    {
        self.task = Some(task.into());
        self
    }

    pub fn auto_merge(&mut self, auto_merge: bool) -> &mut DeploymentOptionsBuilder {
        self.auto_merge = Some(auto_merge);
        self
    }

    pub fn required_contexts<C>(&mut self, ctxs: Vec<C>) -> &mut DeploymentOptionsBuilder
        where C: Into<String>
    {
        self.required_contexts = Some(ctxs.into_iter().map(|c| c.into()).collect::<Vec<String>>());
        self
    }

    pub fn payload<T: serde::ser::Serialize>(&mut self, pl: T) -> &mut DeploymentOptionsBuilder {
        self.payload = serde_json::ser::to_string(&pl).ok();
        self
    }

    pub fn environment<E>(&mut self, env: E) -> &mut DeploymentOptionsBuilder
        where E: Into<String>
    {
        self.environment = Some(env.into());
        self
    }

    pub fn description<D>(&mut self, desc: D) -> &mut DeploymentOptionsBuilder
        where D: Into<String>
    {
        self.description = Some(desc.into());
        self
    }

    pub fn build(&self) -> DeploymentOptions {
        DeploymentOptions {
            commit_ref: self.commit_ref.clone(),
            task: self.task.clone(),
            auto_merge: self.auto_merge,
            required_contexts: self.required_contexts.clone(),
            payload: self.payload.clone(),
            environment: self.environment.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Default)]
pub struct GistListOptions {
    params: HashMap<&'static str, String>,
}

impl GistListOptions {
    pub fn since<T>(timestamp: T) -> GistListOptions
        where T: Into<String>
    {
        let mut params = HashMap::new();
        params.insert("since", timestamp.into());
        GistListOptions { params: params }
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct DeploymentListOptions {
    params: HashMap<&'static str, String>,
}

impl DeploymentListOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> DeploymentListOptionsBuilder {
        DeploymentListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct DeploymentListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl DeploymentListOptionsBuilder {
    pub fn new() -> DeploymentListOptionsBuilder {
        DeploymentListOptionsBuilder { ..Default::default() }
    }

    pub fn sha<S>(&mut self, s: S) -> &mut DeploymentListOptionsBuilder
        where S: Into<String>
    {
        self.params.insert("sha", s.into());
        self
    }

    pub fn commit_ref<G>(&mut self, r: G) -> &mut DeploymentListOptionsBuilder
        where G: Into<String>
    {
        self.params.insert("ref", r.into());
        self
    }

    pub fn task<T>(&mut self, t: T) -> &mut DeploymentListOptionsBuilder
        where T: Into<String>
    {
        self.params.insert("task", t.into());
        self
    }

    pub fn environment<E>(&mut self, e: E) -> &mut DeploymentListOptionsBuilder
        where E: Into<String>
    {
        self.params.insert("environment", e.into());
        self
    }

    pub fn build(&self) -> DeploymentListOptions {
        DeploymentListOptions { params: self.params.clone() }
    }
}

#[derive(Debug, Deserialize)]
pub struct GistFile {
    pub size: u64,
    pub raw_url: String,
    pub content: Option<String>,
    #[serde(rename="type")]
    pub content_type: String,
    pub truncated: Option<bool>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Gist {
    pub url: String,
    pub forks_url: String,
    pub commits_url: String,
    pub id: String,
    pub description: Option<String>,
    pub public: bool,
    pub owner: User,
    pub user: Option<User>,
    pub files: HashMap<String, GistFile>,
    pub truncated: bool,
    pub comments: u64,
    pub comments_url: String,
    pub html_url: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GistFork {
    pub user: User,
    pub url: String,
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Content {
    #[serde(skip_serializing_if="Option::is_none")]
    pub filename: Option<String>,
    pub content: String,
}

impl Content {
    pub fn new<F, C>(filename: Option<F>, content: C) -> Content
        where F: Into<String>,
              C: Into<String>
    {
        Content {
            filename: filename.map(|f| f.into()),
            content: content.into(),
        }
    }
}

#[derive(Default)]
pub struct GistOptionsBuilder {
    pub description: Option<String>,
    pub public: Option<bool>,
    pub files: HashMap<String, Content>,
}

impl GistOptionsBuilder {
    pub fn new<K, V>(files: HashMap<K, V>) -> GistOptionsBuilder
        where K: Clone + Hash + Eq + Into<String>,
              V: Into<String>
    {
        let mut contents = HashMap::new();
        for (k, v) in files.into_iter() {
            contents.insert(k.into(), Content::new(None as Option<String>, v.into()));
        }
        GistOptionsBuilder { files: contents, ..Default::default() }
    }

    pub fn description<D>(&mut self, desc: D) -> &mut GistOptionsBuilder
        where D: Into<String>
    {
        self.description = Some(desc.into());
        self
    }

    pub fn public(&mut self, p: bool) -> &mut GistOptionsBuilder {
        self.public = Some(p);
        self
    }

    pub fn build(&self) -> GistOptions {
        GistOptions {
            files: self.files.clone(),
            description: self.description.clone(),
            public: self.public,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GistOptions {
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub public: Option<bool>,
    pub files: HashMap<String, Content>,
}

impl GistOptions {
    pub fn new<D, K, V>(desc: Option<D>, public: bool, files: HashMap<K, V>) -> GistOptions
        where D: Into<String>,
              K: Hash + Eq + Into<String>,
              V: Into<String>
    {
        let mut contents = HashMap::new();
        for (k, v) in files.into_iter() {
            contents.insert(k.into(), Content::new(None as Option<String>, v.into()));
        }
        GistOptions {
            description: desc.map(|d| d.into()),
            public: Some(public),
            files: contents,
        }
    }

    pub fn builder<K, V>(files: HashMap<K, V>) -> GistOptionsBuilder
        where K: Clone + Hash + Eq + Into<String>,
              V: Into<String>
    {
        GistOptionsBuilder::new(files)
    }
}

#[derive(Debug, Deserialize)]
pub struct Permissions {
    pub admin: bool,
    pub push: bool,
    pub pull: bool,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub id: u64,
    pub owner: User,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub url: String,
    pub html_url: String,
    pub archive_url: String,
    pub assignees_url: String,
    pub blobs_url: String,
    pub branches_url: String,
    pub clone_url: String,
    pub collaborators_url: String,
    pub comments_url: String,
    pub commits_url: String,
    pub compare_url: String,
    pub contents_url: String,
    pub contributors_url: String,
    pub deployments_url: String,
    pub downloads_url: String,
    pub events_url: String,
    pub forks_url: String,
    pub git_commits_url: String,
    pub git_refs_url: String,
    pub git_tags_url: String,
    pub git_url: String,
    pub hooks_url: String,
    pub issue_comment_url: String,
    pub issue_events_url: String,
    pub issues_url: String,
    pub keys_url: String,
    pub labels_url: String,
    pub languages_url: String,
    pub merges_url: String,
    pub milestones_url: String,
    pub mirror_url: Option<String>,
    pub notifications_url: String,
    pub pulls_url: String,
    pub releases_url: String,
    pub ssh_url: String,
    pub stargazers_url: String,
    pub statuses_url: String,
    pub subscribers_url: String,
    pub subscription_url: String,
    pub svn_url: String,
    pub tags_url: String,
    pub teams_url: String,
    pub trees_url: String,
    pub homepage: Option<String>,
    pub language: Option<String>,
    pub forks_count: u64,
    pub stargazers_count: u64,
    pub watchers_count: u64,
    pub size: u64,
    pub default_branch: String,
    pub open_issues_count: u64,
    pub has_issues: bool,
    pub has_wiki: bool,
    pub has_pages: bool,
    pub has_downloads: bool,
    pub pushed_at: String,
    pub created_at: String,
    pub updated_at: String, // permissions: Permissions
}

impl Repo {
    /// Returns a map containing the
    /// [languages](https://developer.github.com/v3/repos/#list-languages) that the repository is
    /// implemented in.
    ///
    /// The keys are the language names, and the values are the number of bytes of code written in
    /// that language.
    pub fn languages(&self, github: &Github) -> Result<HashMap<String, i64>> {
        let url = Url::parse(&self.languages_url).unwrap();
        let uri: String = url.path().into();
        github.get(&uri)
    }
}


#[derive(Debug, Serialize)]
pub struct RepoOptions {
    pub name: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub has_issues: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub has_wiki: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub has_downloads: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub team_id: Option<i32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub auto_init: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub gitignore_template: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub license_template: Option<String>,
}

#[derive(Default)]
pub struct RepoOptionsBuilder {
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    private: Option<bool>,
    has_issues: Option<bool>,
    has_wiki: Option<bool>,
    has_downloads: Option<bool>,
    team_id: Option<i32>,
    auto_init: Option<bool>,
    gitignore_template: Option<String>,
    license_template: Option<String>,
}

impl RepoOptionsBuilder {
    pub fn new<N>(name: N) -> RepoOptionsBuilder
        where N: Into<String>
    {
        RepoOptionsBuilder { name: name.into(), ..Default::default() }
    }

    pub fn description<D>(&mut self, description: D) -> &mut RepoOptionsBuilder
        where D: Into<String>
    {
        self.description = Some(description.into());
        self
    }

    pub fn homepage<H>(&mut self, homepage: H) -> &mut RepoOptionsBuilder
        where H: Into<String>
    {
        self.homepage = Some(homepage.into());
        self
    }

    pub fn private(&mut self, private: bool) -> &mut RepoOptionsBuilder {
        self.private = Some(private);
        self
    }

    pub fn has_issues(&mut self, has_issues: bool) -> &mut RepoOptionsBuilder {
        self.has_issues = Some(has_issues);
        self
    }

    pub fn has_wiki(&mut self, has_wiki: bool) -> &mut RepoOptionsBuilder {
        self.has_wiki = Some(has_wiki);
        self
    }

    pub fn has_downloads(&mut self, has_downloads: bool) -> &mut RepoOptionsBuilder {
        self.has_downloads = Some(has_downloads);
        self
    }

    pub fn team_id(&mut self, team_id: i32) -> &mut RepoOptionsBuilder {
        self.team_id = Some(team_id);
        self
    }

    pub fn auto_init(&mut self, auto_init: bool) -> &mut RepoOptionsBuilder {
        self.auto_init = Some(auto_init);
        self
    }

    pub fn gitignore_template<GI>(&mut self, gitignore_template: GI) -> &mut RepoOptionsBuilder
        where GI: Into<String>
    {
        self.gitignore_template = Some(gitignore_template.into());
        self
    }

    pub fn license_template<L>(&mut self, license_template: L) -> &mut RepoOptionsBuilder
        where L: Into<String>
    {
        self.license_template = Some(license_template.into());
        self
    }

    pub fn build(&self) -> RepoOptions {
        RepoOptions::new(self.name.as_str(),
                         self.description.clone(),
                         self.homepage.clone(),
                         self.private,
                         self.has_issues,
                         self.has_wiki,
                         self.has_downloads,
                         self.team_id,
                         self.auto_init,
                         self.gitignore_template.clone(),
                         self.license_template.clone())
    }
}

impl RepoOptions {
    pub fn new<N, D, H, GI, L>(name: N,
                               description: Option<D>,
                               homepage: Option<H>,
                               private: Option<bool>,
                               has_issues: Option<bool>,
                               has_wiki: Option<bool>,
                               has_downloads: Option<bool>,
                               team_id: Option<i32>,
                               auto_init: Option<bool>,
                               gitignore_template: Option<GI>,
                               license_template: Option<L>)
                               -> RepoOptions
        where N: Into<String>,
              D: Into<String>,
              H: Into<String>,
              GI: Into<String>,
              L: Into<String>
    {
        RepoOptions {
            name: name.into(),
            description: description.map(|h| h.into()),
            homepage: homepage.map(|h| h.into()),
            private: private,
            has_issues: has_issues,
            has_wiki: has_wiki,
            has_downloads: has_downloads,
            team_id: team_id,
            auto_init: auto_init,
            gitignore_template: gitignore_template.map(|gi| gi.into()),
            license_template: license_template.map(|l| l.into()),
        }
    }

    pub fn builder<N: Into<String>>(name: N) -> RepoOptionsBuilder {
        RepoOptionsBuilder::new(name)
    }
}

#[derive(Debug, Deserialize)]
pub struct RepoDetails {
    pub id: u64,
    pub owner: User,
    pub name: String,
    pub full_name: String, // todo
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    // type (keyword)
    pub site_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct Org {
    pub login: String,
    pub id: u64,
    pub url: String,
    pub repos_url: String,
    pub events_url: String,
    pub hooks_url: String,
    pub issues_url: String,
    pub members_url: String,
    pub public_members_url: String,
    pub avatar_url: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub label: String,
    #[serde(rename="ref")]
    pub commit_ref: String,
    pub sha: String,
    pub user: User, //    pub repo: Option<Repo>,
}

#[derive(Debug, Serialize)]
pub struct LabelOptions {
    pub name: String,
    pub color: String,
}

impl LabelOptions {
    pub fn new<N, C>(name: N, color: C) -> LabelOptions
        where N: Into<String>,
              C: Into<String>
    {
        LabelOptions {
            name: name.into(),
            color: color.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Label {
    pub url: String,
    pub name: String,
    pub color: String,
}

#[derive(Default)]
pub struct PullEditOptionsBuilder {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
}

impl PullEditOptionsBuilder {
    pub fn new() -> PullEditOptionsBuilder {
        PullEditOptionsBuilder { ..Default::default() }
    }

    pub fn title<T>(&mut self, title: T) -> &mut PullEditOptionsBuilder
        where T: Into<String>
    {
        self.title = Some(title.into());
        self
    }

    pub fn body<B>(&mut self, body: B) -> &mut PullEditOptionsBuilder
        where B: Into<String>
    {
        self.body = Some(body.into());
        self
    }

    pub fn state<S>(&mut self, state: S) -> &mut PullEditOptionsBuilder
        where S: Into<String>
    {
        self.state = Some(state.into());
        self
    }

    pub fn build(&self) -> PullEditOptions {
        PullEditOptions {
            title: self.title.clone(),
            body: self.body.clone(),
            state: self.state.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PullEditOptions {
    #[serde(skip_serializing_if="Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    state: Option<String>,
}

impl PullEditOptions {
    // todo represent state as enum
    pub fn new<T, B, S>(title: Option<T>, body: Option<B>, state: Option<S>) -> PullEditOptions
        where T: Into<String>,
              B: Into<String>,
              S: Into<String>
    {
        PullEditOptions {
            title: title.map(|t| t.into()),
            body: body.map(|b| b.into()),
            state: state.map(|s| s.into()),
        }
    }
    pub fn builder() -> PullEditOptionsBuilder {
        PullEditOptionsBuilder::new()
    }
}

#[derive(Debug, Serialize)]
pub struct PullOptions {
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub body: Option<String>,
}

impl PullOptions {
    pub fn new<T, H, BS, B>(title: T, head: H, base: BS, body: Option<B>) -> PullOptions
        where T: Into<String>,
              H: Into<String>,
              BS: Into<String>,
              B: Into<String>
    {
        PullOptions {
            title: title.into(),
            head: head.into(),
            base: base.into(),
            body: body.map(|b| b.into()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FileDiff {
    // sha may be null when file mode changed without contents changing
    pub sha: Option<String>,
    pub filename: String,
    pub status: String,
    pub additions: u64,
    pub deletions: u64,
    pub changes: u64,
    pub blob_url: String,
    pub raw_url: String,
    pub contents_url: String,
    /// patch is typically None for binary files
    pub patch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Pull {
    pub id: u64,
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
    pub issue_url: String,
    pub commits_url: String,
    pub review_comments_url: String,
    pub review_comment_url: String,
    pub comments_url: String,
    pub statuses_url: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,
    pub head: Commit,
    pub base: Commit,
    // links
    pub user: User,
    pub assignee: Option<User>,
    pub assignees: Vec<User>,
    pub merge_commit_sha: Option<String>,
    pub mergeable: Option<bool>,
    pub merged_by: Option<User>,
    pub comments: Option<u64>,
    pub commits: Option<u64>,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub changed_files: Option<u64>,
}

#[derive(Default)]
pub struct UserRepoListOptions {
    params: HashMap<&'static str, String>,
}

impl UserRepoListOptions {
    pub fn builder() -> UserRepoListOptionsBuilder {
        UserRepoListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct UserRepoListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl UserRepoListOptionsBuilder {
    pub fn new() -> UserRepoListOptionsBuilder {
        UserRepoListOptionsBuilder { ..Default::default() }
    }

    pub fn repo_type(&mut self, tpe: RepoType) -> &mut UserRepoListOptionsBuilder {
        self.params.insert("type", tpe.to_string());
        self
    }

    pub fn sort(&mut self, sort: RepoSort) -> &mut UserRepoListOptionsBuilder {
        self.params.insert("sort", sort.to_string());
        self
    }

    pub fn asc(&mut self) -> &mut UserRepoListOptionsBuilder {
        self.direction(SortDirection::Asc)
    }

    pub fn desc(&mut self) -> &mut UserRepoListOptionsBuilder {
        self.direction(SortDirection::Desc)
    }

    pub fn direction(&mut self, direction: SortDirection) -> &mut UserRepoListOptionsBuilder {
        self.params.insert("direction", direction.to_string());
        self
    }

    pub fn build(&self) -> UserRepoListOptions {
        UserRepoListOptions { params: self.params.clone() }
    }
}

#[derive(Default)]
pub struct OrganizationRepoListOptions {
    params: HashMap<&'static str, String>,
}

impl OrganizationRepoListOptions {
    pub fn builder() -> OrganizationRepoListOptionsBuilder {
        OrganizationRepoListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct OrganizationRepoListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl OrganizationRepoListOptionsBuilder {
    pub fn new() -> OrganizationRepoListOptionsBuilder {
        OrganizationRepoListOptionsBuilder { ..Default::default() }
    }

    pub fn repo_type(&mut self, tpe: OrgRepoType) -> &mut OrganizationRepoListOptionsBuilder {
        self.params.insert("type", tpe.to_string());
        self
    }

    pub fn build(&self) -> OrganizationRepoListOptions {
        OrganizationRepoListOptions { params: self.params.clone() }
    }
}

#[derive(Default)]
pub struct RepoListOptions {
    params: HashMap<&'static str, String>,
}

impl RepoListOptions {
    pub fn builder() -> RepoListOptionsBuilder {
        RepoListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct RepoListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl RepoListOptionsBuilder {
    pub fn new() -> RepoListOptionsBuilder {
        RepoListOptionsBuilder { ..Default::default() }
    }

    pub fn visibility(&mut self, vis: RepoVisibility) -> &mut RepoListOptionsBuilder {
        self.params.insert("visibility", vis.to_string());
        self
    }

    pub fn affiliation(&mut self, affiliations: Vec<Affiliation>) -> &mut RepoListOptionsBuilder {
        self.params.insert("affiliation",
                           affiliations.into_iter()
                               .map(|a| a.to_string())
                               .collect::<Vec<String>>()
                               .join(","));
        self
    }

    pub fn repo_type(&mut self, tpe: RepoType) -> &mut RepoListOptionsBuilder {
        self.params.insert("type", tpe.to_string());
        self
    }

    pub fn sort(&mut self, sort: RepoSort) -> &mut RepoListOptionsBuilder {
        self.params.insert("sort", sort.to_string());
        self
    }

    pub fn asc(&mut self) -> &mut RepoListOptionsBuilder {
        self.direction(SortDirection::Asc)
    }

    pub fn desc(&mut self) -> &mut RepoListOptionsBuilder {
        self.direction(SortDirection::Desc)
    }

    pub fn direction(&mut self, direction: SortDirection) -> &mut RepoListOptionsBuilder {
        self.params.insert("direction", direction.to_string());
        self
    }

    pub fn build(&self) -> RepoListOptions {
        RepoListOptions { params: self.params.clone() }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResult<D: ::serde::Deserialize> {
    pub total_count: u64,
    pub incomplete_results: bool,
    pub items: Vec<D>,
}

#[derive(Debug, Deserialize)]
pub struct SearchIssuesItem {
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub state: String,
    pub locked: bool,
    pub assignee: Option<User>,
    pub assignees: Vec<User>,
    pub comments: u64,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub pull_request: Option<PullRequestInfo>,
    pub body: Option<String>,
}

impl SearchIssuesItem {
    /// returns a tuple of (repo owner name, repo name) associated with this issue
    pub fn repo_tuple(&self) -> (String, String) {
        let parsed = url::Url::parse(&self.repository_url).unwrap();
        let mut path = parsed.path().split("/").collect::<Vec<_>>();
        path.reverse();
        (path[0].to_owned(), path[1].to_owned())
    }
}

#[derive(Debug, Deserialize)]
pub struct PullRequestInfo {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
}

#[derive(Default)]
pub struct SearchIssuesOptions {
    params: HashMap<&'static str, String>,
}

impl SearchIssuesOptions {
    pub fn builder() -> SearchIssuesOptionsBuilder {
        SearchIssuesOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

/// https://developer.github.com/v3/search/#search-issues
#[derive(Default)]
pub struct SearchIssuesOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl SearchIssuesOptionsBuilder {
    pub fn new() -> SearchIssuesOptionsBuilder {
        SearchIssuesOptionsBuilder { ..Default::default() }
    }

    pub fn sort(&mut self, sort: SearchIssuesSort) -> &mut SearchIssuesOptionsBuilder {
        self.params.insert("sort", sort.to_string());
        self
    }

    pub fn order(&mut self, direction: SortDirection) -> &mut SearchIssuesOptionsBuilder {
        self.params.insert("order", direction.to_string());
        self
    }

    pub fn build(&self) -> SearchIssuesOptions {
        SearchIssuesOptions { params: self.params.clone() }
    }
}

#[derive(Default)]
pub struct PullListOptions {
    params: HashMap<&'static str, String>,
}

impl PullListOptions {
    pub fn builder() -> PullListOptionsBuilder {
        PullListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct PullListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl PullListOptionsBuilder {
    pub fn new() -> PullListOptionsBuilder {
        PullListOptionsBuilder { ..Default::default() }
    }

    pub fn state(&mut self, state: StdState) -> &mut PullListOptionsBuilder {
        self.params.insert("state", state.to_string());
        self
    }

    pub fn sort(&mut self, sort: IssueSort) -> &mut PullListOptionsBuilder {
        self.params.insert("sort", sort.to_string());
        self
    }

    pub fn direction(&mut self, direction: SortDirection) -> &mut PullListOptionsBuilder {
        self.params.insert("direction", direction.to_string());
        self
    }

    pub fn build(&self) -> PullListOptions {
        PullListOptions { params: self.params.clone() }
    }
}

// todo: simplify with param
#[derive(Default)]
pub struct IssueListOptions {
    params: HashMap<&'static str, String>,
}

impl IssueListOptions {
    pub fn builder() -> IssueListOptionsBuilder {
        IssueListOptionsBuilder::new()
    }

    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

/// a mutable issue list builder
#[derive(Default)]
pub struct IssueListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl IssueListOptionsBuilder {
    pub fn new() -> IssueListOptionsBuilder {
        IssueListOptionsBuilder { ..Default::default() }
    }

    pub fn state(&mut self, state: StdState) -> &mut IssueListOptionsBuilder {
        self.params.insert("state", state.to_string());
        self
    }

    pub fn sort(&mut self, sort: IssueSort) -> &mut IssueListOptionsBuilder {
        self.params.insert("sort", sort.to_string());
        self
    }

    pub fn asc(&mut self) -> &mut IssueListOptionsBuilder {
        self.direction(SortDirection::Asc)
    }

    pub fn desc(&mut self) -> &mut IssueListOptionsBuilder {
        self.direction(SortDirection::Desc)
    }

    pub fn direction(&mut self, direction: SortDirection) -> &mut IssueListOptionsBuilder {
        self.params.insert("direction", direction.to_string());
        self
    }

    pub fn assignee<A>(&mut self, assignee: A) -> &mut IssueListOptionsBuilder
        where A: Into<String>
    {
        self.params.insert("assignee", assignee.into());
        self
    }

    pub fn creator<C>(&mut self, creator: C) -> &mut IssueListOptionsBuilder
        where C: Into<String>
    {
        self.params.insert("creator", creator.into());
        self
    }

    pub fn mentioned<M>(&mut self, mentioned: M) -> &mut IssueListOptionsBuilder
        where M: Into<String>
    {
        self.params.insert("mentioned", mentioned.into());
        self
    }

    pub fn labels<L>(&mut self, labels: Vec<L>) -> &mut IssueListOptionsBuilder
        where L: Into<String>
    {
        self.params.insert("labels",
                           labels.into_iter().map(|l| l.into()).collect::<Vec<_>>().join(","));
        self
    }

    pub fn since<S>(&mut self, since: S) -> &mut IssueListOptionsBuilder
        where S: Into<String>
    {
        self.params.insert("since", since.into());
        self
    }

    pub fn build(&self) -> IssueListOptions {
        IssueListOptions { params: self.params.clone() }
    }
}

#[derive(Debug, Serialize)]
pub struct IssueOptions {
    pub title: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub milestone: Option<u64>,
    pub labels: Vec<String>,
}

impl IssueOptions {
    pub fn new<T, B, A, L>(title: T,
                           body: Option<B>,
                           assignee: Option<A>,
                           milestone: Option<u64>,
                           labels: Vec<L>)
                           -> IssueOptions
        where T: Into<String>,
              B: Into<String>,
              A: Into<String>,
              L: Into<String>
    {
        IssueOptions {
            title: title.into(),
            body: body.map(|b| b.into()),
            assignee: assignee.map(|a| a.into()),
            milestone: milestone,
            labels: labels.into_iter().map(|l| l.into()).collect::<Vec<String>>(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: String,
    pub user: User,
    pub labels: Vec<Label>,
    pub assignee: Option<User>,
    pub locked: bool,
    pub comments: u64,
    pub closed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub url: String,
    pub browser_download_url: String,
    pub id: u64,
    pub name: String,
    pub label: Option<String>,
    pub state: String,
    pub content_type: String,
    pub size: u64,
    pub download_count: u64,
    pub created_at: String,
    pub updated_at: String,
    pub uploader: User,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub url: String,
    pub html_url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub tarball_url: String,
    pub zipball_url: String,
    pub id: u64,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub author: User,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseOptions {
    pub tag_name: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub target_commitish: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub prerelease: Option<bool>,
}

/// builder interface for ReleaseOptions
#[derive(Default)]
pub struct ReleaseOptionsBuilder {
    tag: String,
    commitish: Option<String>,
    name: Option<String>,
    body: Option<String>,
    draft: Option<bool>,
    prerelease: Option<bool>,
}

impl ReleaseOptionsBuilder {
    pub fn new<T>(tag: T) -> ReleaseOptionsBuilder
        where T: Into<String>
    {
        ReleaseOptionsBuilder { tag: tag.into(), ..Default::default() }
    }

    pub fn commitish<C>(&mut self, commit: C) -> &mut ReleaseOptionsBuilder
        where C: Into<String>
    {
        self.commitish = Some(commit.into());
        self
    }

    pub fn name<N>(&mut self, name: N) -> &mut ReleaseOptionsBuilder
        where N: Into<String>
    {
        self.name = Some(name.into());
        self
    }

    pub fn body<B>(&mut self, body: B) -> &mut ReleaseOptionsBuilder
        where B: Into<String>
    {
        self.body = Some(body.into());
        self
    }

    pub fn draft(&mut self, draft: bool) -> &mut ReleaseOptionsBuilder {
        self.draft = Some(draft);
        self
    }

    pub fn prerelease(&mut self, pre: bool) -> &mut ReleaseOptionsBuilder {
        self.prerelease = Some(pre);
        self
    }

    pub fn build(&self) -> ReleaseOptions {
        ReleaseOptions::new(self.tag.as_str(),
                            self.commitish.clone(),
                            self.name.clone(),
                            self.body.clone(),
                            self.draft,
                            self.prerelease)
    }
}

impl ReleaseOptions {
    pub fn new<T, C, N, B>(tag: T,
                           commit: Option<C>,
                           name: Option<N>,
                           body: Option<B>,
                           draft: Option<bool>,
                           prerelease: Option<bool>)
                           -> ReleaseOptions
        where T: Into<String>,
              C: Into<String>,
              N: Into<String>,
              B: Into<String>
    {
        ReleaseOptions {
            tag_name: tag.into(),
            target_commitish: commit.map(|c| c.into()),
            name: name.map(|n| n.into()),
            body: body.map(|b| b.into()),
            draft: draft,
            prerelease: prerelease,
        }
    }

    pub fn builder<T>(tag: T) -> ReleaseOptionsBuilder
        where T: Into<String>
    {
        ReleaseOptionsBuilder::new(tag)
    }
}

#[derive(Debug, Deserialize)]
pub struct DeploymentStatus {
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
    pub state: StatusState,
    pub target_url: Option<String>,
    pub description: Option<String>,
    pub id: u64,
    pub deployment_url: String,
    pub repository_url: String,
    pub creator: User,
}

#[derive(Default)]
pub struct DeploymentStatusOptionsBuilder {
    state: StatusState,
    target_url: Option<String>,
    description: Option<String>,
}

impl DeploymentStatusOptionsBuilder {
    pub fn new(state: StatusState) -> DeploymentStatusOptionsBuilder {
        DeploymentStatusOptionsBuilder { state: state, ..Default::default() }
    }

    pub fn target_url<T>(&mut self, url: T) -> &mut DeploymentStatusOptionsBuilder
        where T: Into<String>
    {
        self.target_url = Some(url.into());
        self
    }

    pub fn description<D>(&mut self, desc: D) -> &mut DeploymentStatusOptionsBuilder
        where D: Into<String>
    {
        self.description = Some(desc.into());
        self
    }

    pub fn build(&self) -> DeploymentStatusOptions {
        DeploymentStatusOptions {
            state: self.state.clone(),
            target_url: self.target_url.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeploymentStatusOptions {
    state: StatusState,
    #[serde(skip_serializing_if="Option::is_none")]
    target_url: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    description: Option<String>,
}

impl DeploymentStatusOptions {
    pub fn builder(state: StatusState) -> DeploymentStatusOptionsBuilder {
        DeploymentStatusOptionsBuilder::new(state)
    }
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub created_at: String,
    pub updated_at: String,
    pub state: StatusState,
    pub target_url: String,
    pub description: String,
    pub id: u64,
    pub url: String,
    pub context: String,
    pub creator: User,
}

#[derive(Debug, Serialize)]
pub struct StatusOptions {
    state: StatusState,
    #[serde(skip_serializing_if="Option::is_none")]
    target_url: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    context: Option<String>,
}

#[derive(Default)]
pub struct StatusBuilder {
    state: StatusState,
    target_url: Option<String>,
    description: Option<String>,
    context: Option<String>,
}

impl StatusBuilder {
    pub fn new(state: StatusState) -> StatusBuilder {
        StatusBuilder { state: state, ..Default::default() }
    }

    pub fn target_url<T>(&mut self, url: T) -> &mut StatusBuilder
        where T: Into<String>
    {
        self.target_url = Some(url.into());
        self
    }

    pub fn description<D>(&mut self, desc: D) -> &mut StatusBuilder
        where D: Into<String>
    {
        self.description = Some(desc.into());
        self
    }

    pub fn context<C>(&mut self, ctx: C) -> &mut StatusBuilder
        where C: Into<String>
    {
        self.context = Some(ctx.into());
        self
    }

    pub fn build(&self) -> StatusOptions {
        StatusOptions::new(self.state.clone(),
                           self.target_url.clone(),
                           self.description.clone(),
                           self.context.clone())
    }
}

impl StatusOptions {
    pub fn new<T, D, C>(state: StatusState,
                        target_url: Option<T>,
                        descr: Option<D>,
                        context: Option<C>)
                        -> StatusOptions
        where T: Into<String>,
              D: Into<String>,
              C: Into<String>
    {
        StatusOptions {
            state: state,
            target_url: target_url.map(|t| t.into()),
            description: descr.map(|d| d.into()),
            context: context.map(|c| c.into()),
        }
    }

    pub fn builder(state: StatusState) -> StatusBuilder {
        StatusBuilder::new(state)
    }
}

#[derive(Debug, Deserialize)]
pub struct Key {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub verified: bool,
    pub created_at: String,
    pub read_only: bool,
}

#[derive(Debug, Serialize)]
pub struct KeyOptions {
    pub title: String,
    pub key: String,
    pub read_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReviewComment {
    pub id: u64,
    pub url: String,
    pub diff_hunk: String,
    pub path: String,
    pub position: u64,
    pub original_position: u64,
    pub commit_id: String,
    pub original_commit_id: String,
    pub user: User,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub pull_request_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PullCommit {
    pub url: String,
    pub sha: String,
    pub html_url: String,
    pub comments_url: String,
    pub commit: CommitDetails,
    pub author: User,
    pub committer: User,
    pub parents: Vec<CommitRef>,
}

#[derive(Debug, Deserialize)]
pub struct CommitDetails {
    pub url: String,
    pub author: UserStamp,
    pub committer: Option<UserStamp>,
    pub message: String,
    pub tree: CommitRef,
    pub comment_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct UserStamp {
    pub name: String,
    pub email: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitRef {
    pub url: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub url: String,
    pub html_url: String,
    pub body: String,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Default)]
pub struct CommentListOptions {
    params: HashMap<&'static str, String>,
}

impl CommentListOptions {
    pub fn builder() -> CommentListOptionsBuilder {
        CommentListOptionsBuilder::new()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}

#[derive(Default)]
pub struct CommentListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl CommentListOptionsBuilder {
    pub fn new() -> CommentListOptionsBuilder {
        CommentListOptionsBuilder { ..Default::default() }
    }

    pub fn since<S>(&mut self, since: S) -> &mut CommentListOptionsBuilder
        where S: Into<String>
    {
        self.params.insert("since", since.into());
        self
    }

    pub fn build(&self) -> CommentListOptions {
        CommentListOptions { params: self.params.clone() }
    }
}

/// options for creating a repository hook
/// see [this](https://developer.github.com/v3/repos/hooks/#create-a-hook)
/// for githubs official documentation
#[derive(Debug, Serialize)]
pub struct HookCreateOptions {
    name: String,
    config: BTreeMap<String, ::serde_json::Value>,
    events: Vec<String>,
    active: bool,
}

impl HookCreateOptions {
    /// creates a new builder instance with a hook name
    /// are should be taken with respect to the hook name as you can only
    /// use "web" or the a valid service name listed [here](https://api.github.com/hooks)
    pub fn builder<N>(name: N) -> HookCreateOptionsBuilder
        where N: Into<String>
    {
        HookCreateOptionsBuilder::new(name)
    }

    /// use this for creating a builder for webhook options
    pub fn web() -> HookCreateOptionsBuilder {
        Self::builder("web")
    }
}

#[derive(Default)]
pub struct HookCreateOptionsBuilder {
    name: String,
    config: BTreeMap<String, ::serde_json::Value>,
    events: Vec<String>,
    active: bool,
}

impl HookCreateOptionsBuilder {
    pub fn new<N>(name: N) -> HookCreateOptionsBuilder
        where N: Into<String>
    {
        HookCreateOptionsBuilder {
            name: name.into(),
            active: true,
            ..Default::default()
        }
    }


    pub fn active(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }

    /// a list of github events this hook should receive deliveries for
    /// the default is "push". for a full list, see
    /// the [Github api docs](https://developer.github.com/webhooks/#events)
    pub fn events<E>(&mut self, events: Vec<E>) -> &mut Self
        where E: Into<String>
    {
        self.events = events.into_iter().map(|e| e.into()).collect::<Vec<_>>();
        self
    }

    /// web hooks must have an associated url
    pub fn url<U>(&mut self, url: U) -> &mut Self
        where U: Into<String>
    {
        self.config_entry("url".to_owned(), ::serde_json::Value::String(url.into()))
    }

    /// web hooks can optionally specify a content_type of "form" or "json"
    /// which indicates the type of payload they will expect to receive
    pub fn content_type(&mut self, content_type: WebHookContentType) -> &mut Self {
        self.config_str_entry("content_type", content_type.to_string());
        self
    }

    /// web hooks can optionally provide a secret used to sign deliveries
    /// to identify that their source was indeed github
    pub fn secret<S>(&mut self, sec: S) -> &mut Self
        where S: Into<String>
    {
        self.config_str_entry("secret", sec);
        self
    }

    pub fn config_str_entry<K, V>(&mut self, k: K, v: V) -> &mut Self
        where K: Into<String>,
              V: Into<String>
    {
        self.config_entry(k.into(), ::serde_json::Value::String(v.into()));
        self
    }

    pub fn config_entry<N>(&mut self, name: N, value: ::serde_json::Value) -> &mut Self
        where N: Into<String>
    {
        self.config.insert(name.into(), value);
        self
    }

    pub fn build(&self) -> HookCreateOptions {
        HookCreateOptions {
            name: self.name.clone(),
            config: self.config.clone(),
            events: self.events.clone(),
            active: self.active,
        }
    }
}


/// options for editing a repository hook
/// see [this](https://developer.github.com/v3/repos/hooks/#edit-a-hook)
/// for githubs official documentation
#[derive(Debug, Serialize)]
pub struct HookEditOptions {
    config: BTreeMap<String, ::serde_json::Value>,
    events: Vec<String>,
    add_events: Vec<String>,
    remove_events: Vec<String>,
    active: bool,
}

impl HookEditOptions {
    /// creates a new builder instance
    pub fn builder() -> HookEditOptionsBuilder {
        HookEditOptionsBuilder::new()
    }
}

#[derive(Default)]
pub struct HookEditOptionsBuilder {
    config: BTreeMap<String, ::serde_json::Value>,
    events: Vec<String>,
    add_events: Vec<String>,
    remove_events: Vec<String>,
    active: bool,
}

impl HookEditOptionsBuilder {
    pub fn new() -> HookEditOptionsBuilder {
        HookEditOptionsBuilder { ..Default::default() }
    }


    pub fn active(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }

    /// a list of github events this hook should receive deliveries for
    /// the default is "push". for a full list, see
    /// the [Github api docs](https://developer.github.com/webhooks/#events)
    pub fn events<E>(&mut self, events: Vec<E>) -> &mut Self
        where E: Into<String>
    {
        self.events = events.into_iter().map(|e| e.into()).collect::<Vec<_>>();
        self
    }

    /// web hooks must have an associated url
    pub fn url<U>(&mut self, url: U) -> &mut Self
        where U: Into<String>
    {
        self.config_entry("url".to_owned(), ::serde_json::Value::String(url.into()))
    }

    /// web hooks can optionally specify a content_type of "form" or "json"
    /// which indicates the type of payload they will expect to receive
    pub fn content_type(&mut self, content_type: WebHookContentType) -> &mut Self {
        self.config_str_entry("content_type", content_type.to_string());
        self
    }

    /// web hooks can optionally provide a secret used to sign deliveries
    /// to identify that their source was indeed github
    pub fn secret<S>(&mut self, sec: S) -> &mut Self
        where S: Into<String>
    {
        self.config_str_entry("secret", sec);
        self
    }

    pub fn config_str_entry<K, V>(&mut self, k: K, v: V) -> &mut Self
        where K: Into<String>,
              V: Into<String>
    {
        self.config_entry(k.into(), ::serde_json::Value::String(v.into()));
        self
    }

    pub fn config_entry<N>(&mut self, name: N, value: ::serde_json::Value) -> &mut Self
        where N: Into<String>
    {
        self.config.insert(name.into(), value);
        self
    }

    pub fn build(&self) -> HookEditOptions {
        HookEditOptions {
            config: self.config.clone(),
            events: self.events.clone(),
            add_events: self.add_events.clone(),
            remove_events: self.remove_events.clone(),
            active: self.active,
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct Hook {
    pub id: u64,
    pub url: String,
    pub test_url: String,
    pub ping_url: String,
    pub name: String,
    pub events: Vec<String>,
    pub config: ::serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub active: bool,
}

impl Hook {
    pub fn config_value(&self, name: &str) -> Option<&::serde_json::Value> {
        self.config.pointer(&format!("/{}", name))
    }

    pub fn config_string(&self, name: &str) -> Option<String> {
        self.config_value(name).and_then(|value| match *value {
            ::serde_json::Value::String(ref val) => Some(val.clone()),
            _ => None,
        })
    }

    pub fn url(&self) -> Option<String> {
        self.config_string("url")
    }

    pub fn content_type(&self) -> Option<String> {
        self.config_string("content_type")
    }
}


#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum StatusState {
    /// pending
    #[serde(rename = "pending")]
    Pending,
    /// success
    #[serde(rename = "success")]
    Success,
    /// error
    #[serde(rename = "error")]
    Error,
    /// failure
    #[serde(rename = "failure")]
    Failure,
}

impl Default for StatusState {
    fn default() -> StatusState {
        StatusState::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::super::State as StdState;
    use serde::ser::Serialize;
    use std::collections::{HashMap, BTreeMap};
    use super::*;
    use super::serde_json;

    fn test_encoding<E: Serialize>(tests: Vec<(E, &str)>) {
        for test in tests {
            match test {
                (k, v) => assert_eq!(serde_json::to_string(&k).unwrap(), v),
            }
        }
    }

    #[test]
    fn gist_reqs() {
        let mut files = HashMap::new();
        files.insert("foo", "bar");
        let tests = vec![(GistOptions::new(None as Option<String>, true, files.clone()),
                  r#"{"public":true,"files":{"foo":{"content":"bar"}}}"#),
                 (GistOptions::new(Some("desc"), true, files.clone()),
                  r#"{"description":"desc","public":true,"files":{"foo":{"content":"bar"}}}"#)];
        test_encoding(tests);
    }

    #[test]
    fn deserialize_client_field_errors() {
        for (json, expect) in vec![// see https://github.com/softprops/hubcaps/issues/31
                                   (r#"{"message": "Validation Failed","errors":
                [{
                    "resource": "Release",
                    "code": "custom",
                    "message": "Published releases must have a valid tag"
                }]}"#,
                                    ClientError {
                                       message: "Validation Failed".to_owned(),
                                       errors: Some(vec![FieldErr {
                                                             resource: "Release".to_owned(),
                                                             code: "custom".to_owned(),
                                                             field: None,
                                                             message: Some("Published releases \
                                                                            must have a valid tag"
                                                                 .to_owned()),
                                                             documentation_url: None,
                                                         }]),
                                   })] {
            assert_eq!(serde_json::from_str::<ClientError>(json).unwrap(), expect);
        }
    }

    #[test]
    fn deserialize_status_state() {
        for (json, value) in vec![("\"pending\"", StatusState::Pending),
                                   ("\"success\"", StatusState::Success),
                                   ("\"error\"", StatusState::Error),
                                   ("\"failure\"", StatusState::Failure)] {
            assert_eq!(serde_json::from_str::<StatusState>(json).unwrap(), value)
        }
    }

    #[test]
    fn serialize_status_state() {
        for (json, value) in vec![("\"pending\"", StatusState::Pending),
                                  ("\"success\"", StatusState::Success),
                                  ("\"error\"", StatusState::Error),
                                  ("\"failure\"", StatusState::Failure)] {
            assert_eq!(serde_json::to_string(&value).unwrap(), json)
        }
    }

    #[test]
    fn hook_create_reqs() {}

    #[test]
    fn hook_edit_reqs() {}

    #[test]
    fn deployment_reqs() {
        let mut payload = BTreeMap::new();
        payload.insert("user", "atmos");
        payload.insert("room_id", "123456");
        let tests = vec![
            (
                DeploymentOptions::builder("test").build(),
                r#"{"ref":"test"}"#
            ),
            (
                DeploymentOptions::builder("test").task("launchit").build(),
                r#"{"ref":"test","task":"launchit"}"#
            ),
            (
                DeploymentOptions::builder("topic-branch")
                    .description("description")
                    .payload(payload)
                    .build(),
                r#"{"ref":"topic-branch","payload":"{\"room_id\":\"123456\",\"user\":\"atmos\"}","description":"description"}"#
            )
        ];
        test_encoding(tests)
    }

    #[test]
    fn deployment_status_reqs() {
        let tests = vec![
            (
                DeploymentStatusOptions::builder(StatusState::Pending)
                    .build(),
                r#"{"state":"pending"}"#
            ),
            (
                DeploymentStatusOptions::builder(StatusState::Pending)
                    .target_url("http://host.com")
                    .build(),
                r#"{"state":"pending","target_url":"http://host.com"}"#
            ),
            (
                DeploymentStatusOptions::builder(StatusState::Pending)
                    .target_url("http://host.com")
                    .description("desc")
                    .build(),
                r#"{"state":"pending","target_url":"http://host.com","description":"desc"}"#
            ),
        ];
        test_encoding(tests)
    }

    #[test]
    fn gist_req() {
        let mut files = HashMap::new();
        files.insert("test", "foo");
        let tests = vec![(GistOptions::builder(files.clone()).build(),
                  r#"{"files":{"test":{"content":"foo"}}}"#),
                 (GistOptions::builder(files.clone()).description("desc").build(),
                  r#"{"description":"desc","files":{"test":{"content":"foo"}}}"#),
                 (GistOptions::builder(files.clone()).description("desc").public(false).build(),
                  r#"{"description":"desc","public":false,"files":{"test":{"content":"foo"}}}"#)];
        test_encoding(tests)
    }

    #[test]
    fn pullreq_edits() {
        let tests = vec![(PullEditOptions::builder().title("test").build(), r#"{"title":"test"}"#),
                         (PullEditOptions::builder().title("test").body("desc").build(),
                          r#"{"title":"test","body":"desc"}"#),
                         (PullEditOptions::builder().state("closed").build(),
                          r#"{"state":"closed"}"#)];
        test_encoding(tests)
    }

    #[test]
    fn status_reqs() {
        let tests = vec![(StatusOptions::builder(StatusState::Pending).build(),
                  r#"{"state":"pending"}"#),
                 (StatusOptions::builder(StatusState::Success)
                      .target_url("http://acme.com")
                      .build(),
                  r#"{"state":"success","target_url":"http://acme.com"}"#),
                 (StatusOptions::builder(StatusState::Error).description("desc").build(),
                  r#"{"state":"error","description":"desc"}"#),
                 (StatusOptions::builder(StatusState::Failure)
                      .target_url("http://acme.com")
                      .description("desc")
                      .build(),
                  r#"{"state":"failure","target_url":"http://acme.com","description":"desc"}"#)];
        test_encoding(tests)
    }

    #[test]
    fn issue_list_reqs() {
        fn test_serialize(tests: Vec<(IssueListOptions, Option<String>)>) {
            for test in tests {
                match test {
                    (k, v) => assert_eq!(k.serialize(), v),
                }
            }
        }
        let tests = vec![
            (
                IssueListOptions::builder().build(),
                None
            ),
            (
                IssueListOptions::builder().state(StdState::Closed).build(),
                Some("state=closed".to_owned())
             ),
            (
                IssueListOptions::builder().labels(vec!["foo", "bar"]).build(),
                Some("labels=foo%2Cbar".to_owned())
            ),
        ];
        test_serialize(tests)
    }

    #[test]
    fn pull_list_reqs() {
        fn test_serialize(tests: Vec<(PullListOptions, Option<String>)>) {
            for test in tests {
                match test {
                    (k, v) => assert_eq!(k.serialize(), v),
                }
            }
        }
        let tests = vec![(PullListOptions::builder().build(), None),
                         (PullListOptions::builder().state(StdState::Closed).build(),
                          Some("state=closed".to_owned()))];
        test_serialize(tests)
    }
}
