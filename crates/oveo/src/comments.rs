//! Comment-based metadata annotations.
//!
//! Substring matching: any comment in leading position whose content
//! contains one of:
//!
//! - `@__HOIST__` — hoist the following expression
//! - `@__CONST__` — mark the following expression for deduplication
//! - `@__SCOPE__` — mark the following arrow/function expression as a new
//!   hoist scope
//!
//! Both block and line comments are recognized. If a single comment contains
//! multiple markers, the first match in the order Hoist, Scope, Const wins.

use oxc_ast::ast::Comment;
use rustc_hash::FxHashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommentAnnotation {
    Hoist,
    Scope,
    Const,
}

/// Marker substring for hoist annotations.
pub const HOIST_TEXT: &str = "@__HOIST__";
/// Marker substring for const (dedupe) annotations.
pub const CONST_TEXT: &str = "@__CONST__";
/// Marker substring for scope annotations.
pub const SCOPE_TEXT: &str = "@__SCOPE__";

pub fn annotation_for_content(content: &str) -> Option<CommentAnnotation> {
    if content.contains(HOIST_TEXT) {
        Some(CommentAnnotation::Hoist)
    } else if content.contains(SCOPE_TEXT) {
        Some(CommentAnnotation::Scope)
    } else if content.contains(CONST_TEXT) {
        Some(CommentAnnotation::Const)
    } else {
        None
    }
}

pub fn is_annotation_content(content: &str) -> bool {
    annotation_for_content(content).is_some()
}

/// Builds a map from `attached_to` (start offset of the following token) to
/// the annotation. Only leading comments whose content contains a marker
/// substring are included.
pub fn build_comment_annotations(
    source_text: &str,
    comments: &[Comment],
) -> FxHashMap<u32, CommentAnnotation> {
    let mut map = FxHashMap::default();
    for comment in comments {
        if !comment.is_leading() {
            continue;
        }
        let content = comment.content_span().source_text(source_text);
        if let Some(annotation) = annotation_for_content(content) {
            map.insert(comment.attached_to, annotation);
        }
    }
    map
}
