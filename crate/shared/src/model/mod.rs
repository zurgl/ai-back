use crate::config::path;

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Models {
    pub models: Vec<Model>,
}

impl IntoIterator for Models {
    type Item = Model;
    type IntoIter = <Vec<Model> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.models.into_iter()
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Model {
    pub name: String,
    pub ressources: Vec<Ressource>,
}

impl Model {
    pub fn is_valid_ressource(&self, ressource_name: &str) -> bool {
        self.ressources
            .iter()
            .find(|ressource| ressource.name == ressource_name)
            .and(Some(true))
            .unwrap_or(false)
    }
}

impl Models {
    pub fn is_valid_model(&self, model_name: &str) -> bool {
        self.models
            .iter()
            .filter(|model| model.name == model_name)
            .count()
            > 0
    }

    pub fn load() -> Models {
        ron::from_str(
            &std::fs::read_to_string(path::config().join("ressources.ron"))
                .expect("Cannot read the file"),
        )
        .unwrap()
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Ressource {
    pub url: String,
    pub deflate: bool,
    pub name: String,
    pub h2: bool,
}
