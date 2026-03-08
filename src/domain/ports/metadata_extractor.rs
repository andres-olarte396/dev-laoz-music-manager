use crate::domain::entities::track::Track;

pub trait MetadataExtractor {
    /// Devuelve un `Track` extraido con Metadata rica.
    /// Si falla por corrupción, el adaptador debe manejar el error y retornar 
    /// un `Track::fallback()` garantizando que la entidad nunca se pierda en el disco duro.
    fn extract_metadata(&self, file_path: &str) -> Track;
}
