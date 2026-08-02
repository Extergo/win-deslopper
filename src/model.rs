//! Typed mock catalogue and local preview-plan state.

use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ComponentId {
    OneDrive,
    Copilot,
    PromotionalContent,
}

impl ComponentId {
    pub fn key(self) -> &'static str {
        match self {
            Self::OneDrive => "onedrive",
            Self::Copilot => "copilot",
            Self::PromotionalContent => "promotional_content",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "onedrive" => Some(Self::OneDrive),
            "copilot" => Some(Self::Copilot),
            "promotional_content" => Some(Self::PromotionalContent),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentCategory {
    CloudIntegration,
    AiAndAssistantFeatures,
    RecommendationsAndPromotions,
}

impl ComponentCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::CloudIntegration => "Cloud integration",
            Self::AiAndAssistantFeatures => "AI and assistant features",
            Self::RecommendationsAndPromotions => "Recommendations and promotions",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Low,
    Moderate,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low risk",
            Self::Moderate => "Moderate risk",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartRequirement {
    NotExpected,
    MayBeRequired,
}

impl RestartRequirement {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotExpected => "Restart not expected",
            Self::MayBeRequired => "May require restart",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityState {
    SupportedOptionsVary,
    VariesByBuildAndRegion,
    SettingDependent,
}

impl CompatibilityState {
    pub fn label(self) -> &'static str {
        match self {
            Self::SupportedOptionsVary => "Supported options vary",
            Self::VariesByBuildAndRegion => "Varies by build and region",
            Self::SettingDependent => "Availability varies by settings",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogueFilter {
    All,
    Cloud,
    Ai,
    Promotions,
}

impl CatalogueFilter {
    pub fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Cloud => "cloud",
            Self::Ai => "ai",
            Self::Promotions => "promotions",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "all" => Some(Self::All),
            "cloud" => Some(Self::Cloud),
            "ai" => Some(Self::Ai),
            "promotions" => Some(Self::Promotions),
            _ => None,
        }
    }

    fn matches(self, category: ComponentCategory) -> bool {
        match self {
            Self::All => true,
            Self::Cloud => category == ComponentCategory::CloudIntegration,
            Self::Ai => category == ComponentCategory::AiAndAssistantFeatures,
            Self::Promotions => category == ComponentCategory::RecommendationsAndPromotions,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationDestination {
    WindowsCleanup,
    InspectionHistory,
    DriftHistory,
    Extensions,
}

impl NavigationDestination {
    pub fn key(self) -> &'static str {
        match self {
            Self::WindowsCleanup => "windows_cleanup",
            Self::InspectionHistory => "inspection_history",
            Self::DriftHistory => "drift_history",
            Self::Extensions => "extensions",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "windows_cleanup" => Some(Self::WindowsCleanup),
            "inspection_history" => Some(Self::InspectionHistory),
            "drift_history" => Some(Self::DriftHistory),
            "extensions" => Some(Self::Extensions),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupComponent {
    pub id: ComponentId,
    pub name: &'static str,
    pub description: &'static str,
    pub category: ComponentCategory,
    pub risk: RiskLevel,
    pub restart: RestartRequirement,
    pub compatibility: CompatibilityState,
    pub proposed_actions: &'static str,
}

pub fn mock_catalogue() -> Vec<CleanupComponent> {
    vec![
        CleanupComponent {
            id: ComponentId::OneDrive,
            name: "Microsoft OneDrive",
            description: "Cloud file syncing integrated into Windows and File Explorer. Supported options depend on how OneDrive was installed and configured.",
            category: ComponentCategory::CloudIntegration,
            risk: RiskLevel::Moderate,
            restart: RestartRequirement::MayBeRequired,
            compatibility: CompatibilityState::SupportedOptionsVary,
            proposed_actions: "Future options: disable startup, disconnect integration, or uninstall a supported application package where applicable.",
        },
        CleanupComponent {
            id: ComponentId::Copilot,
            name: "Windows Copilot",
            description: "Assistant experiences whose availability and implementation vary across Windows versions, regions, accounts, and builds.",
            category: ComponentCategory::AiAndAssistantFeatures,
            risk: RiskLevel::Low,
            restart: RestartRequirement::NotExpected,
            compatibility: CompatibilityState::VariesByBuildAndRegion,
            proposed_actions: "Future option: hide or disable supported Copilot surfaces only where Windows provides a documented mechanism.",
        },
        CleanupComponent {
            id: ComponentId::PromotionalContent,
            name: "Windows promotional content",
            description: "Promotional suggestions, recommendations, consumer experiences, suggested apps, and advertising-like surfaces in Windows.",
            category: ComponentCategory::RecommendationsAndPromotions,
            risk: RiskLevel::Low,
            restart: RestartRequirement::NotExpected,
            compatibility: CompatibilityState::SettingDependent,
            proposed_actions: "Future options: reduce promotional suggestions and disable supported recommendation or consumer-content settings.",
        },
    ]
}

pub struct AppState {
    catalogue: Vec<CleanupComponent>,
    query: String,
    filter: CatalogueFilter,
    planned: HashSet<ComponentId>,
    destination: NavigationDestination,
    review_open: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            catalogue: mock_catalogue(),
            query: String::new(),
            filter: CatalogueFilter::All,
            planned: HashSet::new(),
            destination: NavigationDestination::WindowsCleanup,
            review_open: false,
        }
    }

    pub fn catalogue_len(&self) -> usize {
        self.catalogue.len()
    }

    pub fn filter(&self) -> CatalogueFilter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: CatalogueFilter) {
        self.filter = filter;
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
    }

    pub fn destination(&self) -> NavigationDestination {
        self.destination
    }

    pub fn set_destination(&mut self, destination: NavigationDestination) {
        self.destination = destination;
    }

    pub fn is_review_open(&self) -> bool {
        self.review_open
    }

    pub fn open_review(&mut self) {
        if !self.planned.is_empty() {
            self.review_open = true;
        }
    }

    pub fn close_review(&mut self) {
        self.review_open = false;
    }

    pub fn toggle_planned(&mut self, id: ComponentId) {
        if !self.planned.remove(&id) {
            self.planned.insert(id);
        }
        if self.planned.is_empty() {
            self.review_open = false;
        }
    }

    pub fn is_planned(&self, id: ComponentId) -> bool {
        self.planned.contains(&id)
    }

    pub fn planned_count(&self) -> usize {
        self.planned.len()
    }

    pub fn visible_components(&self) -> Vec<&CleanupComponent> {
        let query = self.query.trim().to_lowercase();
        self.catalogue
            .iter()
            .filter(|component| self.filter.matches(component.category))
            .filter(|component| {
                query.is_empty()
                    || component.name.to_lowercase().contains(&query)
                    || component.category.label().to_lowercase().contains(&query)
                    || component.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn planned_components(&self) -> Vec<&CleanupComponent> {
        self.catalogue
            .iter()
            .filter(|component| self.is_planned(component.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_matches_name_category_and_description() {
        let mut state = AppState::new();

        state.set_query("onedrive");
        assert_eq!(state.visible_components()[0].id, ComponentId::OneDrive);

        state.set_query("assistant features");
        assert_eq!(state.visible_components()[0].id, ComponentId::Copilot);

        state.set_query("suggested apps");
        assert_eq!(
            state.visible_components()[0].id,
            ComponentId::PromotionalContent
        );
    }

    #[test]
    fn category_filter_limits_catalogue() {
        let mut state = AppState::new();
        state.set_filter(CatalogueFilter::Cloud);

        let visible = state.visible_components();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].category, ComponentCategory::CloudIntegration);
    }

    #[test]
    fn planning_adds_removes_and_counts_items() {
        let mut state = AppState::new();

        state.toggle_planned(ComponentId::Copilot);
        assert!(state.is_planned(ComponentId::Copilot));
        assert_eq!(state.planned_count(), 1);

        state.toggle_planned(ComponentId::OneDrive);
        assert_eq!(state.planned_count(), 2);

        state.toggle_planned(ComponentId::Copilot);
        assert!(!state.is_planned(ComponentId::Copilot));
        assert_eq!(state.planned_count(), 1);
    }

    #[test]
    fn review_only_opens_for_a_non_empty_plan() {
        let mut state = AppState::new();
        state.open_review();
        assert!(!state.is_review_open());

        state.toggle_planned(ComponentId::OneDrive);
        state.open_review();
        assert!(state.is_review_open());

        state.toggle_planned(ComponentId::OneDrive);
        assert!(!state.is_review_open());
    }

    #[test]
    fn enum_display_mappings_are_precise() {
        assert_eq!(RiskLevel::Low.label(), "Low risk");
        assert_eq!(
            RestartRequirement::MayBeRequired.label(),
            "May require restart"
        );
        assert_eq!(
            CompatibilityState::VariesByBuildAndRegion.label(),
            "Varies by build and region"
        );
        assert_eq!(
            ComponentId::from_key(ComponentId::OneDrive.key()),
            Some(ComponentId::OneDrive)
        );
        assert_eq!(
            CatalogueFilter::from_key(CatalogueFilter::Promotions.key()),
            Some(CatalogueFilter::Promotions)
        );
        assert_eq!(
            NavigationDestination::from_key(NavigationDestination::Extensions.key()),
            Some(NavigationDestination::Extensions)
        );
    }
}
