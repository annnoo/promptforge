use prompt_core::PromptDocument;

const DEFAULT_MAX_HISTORY: usize = 100;

/// Snapshot-based undo/redo history manager for `PromptDocument`.
#[derive(Debug, Clone)]
pub struct DocumentHistory {
    past: Vec<PromptDocument>,
    future: Vec<PromptDocument>,
    max_depth: usize,
}

impl Default for DocumentHistory {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_HISTORY)
    }
}

impl DocumentHistory {
    /// Creates a new history tracker with a maximum snapshot depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_depth: if max_depth == 0 { DEFAULT_MAX_HISTORY } else { max_depth },
        }
    }

    /// Pushes the current document state as an undo snapshot.
    /// Clears any redo future since a new edit occurred.
    pub fn push_snapshot(&mut self, current: PromptDocument) {
        // Avoid pushing identical snapshot if top of stack is identical
        if let Some(last) = self.past.last() {
            if last == &current {
                return;
            }
        }
        self.past.push(current);
        if self.past.len() > self.max_depth {
            self.past.remove(0);
        }
        self.future.clear();
    }

    /// Checks if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Checks if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Undoes to the previous document state, saving the current state into the redo future.
    pub fn undo(&mut self, current: PromptDocument) -> Option<PromptDocument> {
        let prev = self.past.pop()?;
        self.future.push(current);
        Some(prev)
    }

    /// Redoes to the next document state, saving the current state into the undo past.
    pub fn redo(&mut self, current: PromptDocument) -> Option<PromptDocument> {
        let next = self.future.pop()?;
        self.past.push(current);
        Some(next)
    }

    /// Clears all undo and redo history.
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_flow() {
        let mut history = DocumentHistory::default();
        let doc0 = PromptDocument::new("Doc 0", "");
        let mut doc1 = doc0.clone();
        doc1.title = "Doc 1".to_string();
        let mut doc2 = doc1.clone();
        doc2.title = "Doc 2".to_string();

        assert!(!history.can_undo());
        assert!(!history.can_redo());

        // Snapshot doc0 before going to doc1
        history.push_snapshot(doc0.clone());
        // Snapshot doc1 before going to doc2
        history.push_snapshot(doc1.clone());

        assert!(history.can_undo());

        // Undo from doc2 should restore doc1
        let restored_1 = history.undo(doc2.clone()).unwrap();
        assert_eq!(restored_1.title, "Doc 1");
        assert!(history.can_redo());

        // Undo from doc1 should restore doc0
        let restored_0 = history.undo(restored_1.clone()).unwrap();
        assert_eq!(restored_0.title, "Doc 0");

        // Redo should return doc1
        let redone_1 = history.redo(restored_0).unwrap();
        assert_eq!(redone_1.title, "Doc 1");

        // Redo should return doc2
        let redone_2 = history.redo(redone_1).unwrap();
        assert_eq!(redone_2.title, "Doc 2");
        assert!(!history.can_redo());
    }

    #[test]
    fn test_identical_snapshots_deduplicated() {
        let mut history = DocumentHistory::default();
        let doc = PromptDocument::new("Doc", "");
        history.push_snapshot(doc.clone());
        history.push_snapshot(doc.clone());
        assert_eq!(history.past.len(), 1);
    }
}
