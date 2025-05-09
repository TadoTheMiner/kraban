use crossterm::event::{KeyCode, KeyEvent};
use kraban_state::{Difficulty, Priority};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
};
use tap::Tap;

use crate::{
    Action, Component, Context, Item,
    keyhints::KeyHints,
    list::{ListState, WrappingUsize},
    main_view::MainView,
    open_prompt,
    prompt::{
        DeleteConfirmation, DueDatePrompt, EnumPrompt, InputAction, InputPrompt, MoveToColumnPrompt,
    },
    switch_to_view,
};

use super::TasksView;

impl TasksView {
    fn on_key_mutable(&mut self, key_event: KeyEvent, context: Context) -> Option<Action> {
        match key_event.code {
            KeyCode::Delete | KeyCode::Backspace => {
                self.focused_task.focused_item().and_then(|index| {
                    open_prompt(DeleteConfirmation {
                        name: self.current_task(context, index).title,
                        item: Item::Task,
                    })
                })
            }
            KeyCode::Char('n') => open_prompt(InputPrompt::new(
                context,
                InputAction::New,
                "Enter new task name".to_string(),
            )),
            // TODO: This is both in task and project and therefore violates DRY, fix that
            KeyCode::Char('p') => self.focused_task.focused_item().and_then(|index| {
                open_prompt({
                    let priority_prompt: EnumPrompt<Priority> =
                        EnumPrompt::new(self.current_task(context, index).priority);
                    priority_prompt
                })
            }),
            KeyCode::Char('d') => self.focused_task.focused_item().and_then(|index| {
                open_prompt({
                    let difficulty_prompt: EnumPrompt<Difficulty> =
                        EnumPrompt::new(self.current_task(context, index).difficulty);
                    difficulty_prompt
                })
            }),
            KeyCode::Char('r') => self.focused_task.focused_item().and_then(|index| {
                open_prompt(InputPrompt::new(
                    context,
                    InputAction::Rename,
                    self.current_task(context, index).title,
                ))
            }),
            KeyCode::Char('a') => self.focused_task.focused_item().and_then(|index| {
                open_prompt(DueDatePrompt::new(
                    self.current_task(context, index).due_date,
                ))
            }),
            _ => self.focused_task.on_key(key_event, context),
        }
    }

    fn reset_focused_task(&mut self, context: Context) {
        self.focused_task = ListState::new(self.get_current_column_len(context).checked_sub(1));
    }

    fn reset_focused_column(&mut self, context: Context) {
        self.focused_column = WrappingUsize::new(
            context.config.tabs[usize::from(self.focused_tab)]
                .columns
                .len()
                - 1,
        );
        self.reset_focused_task(context);
    }
}

impl Component for TasksView {
    fn on_key(&mut self, key_event: KeyEvent, context: Context) -> Option<Action> {
        match key_event.code {
            KeyCode::Left => {
                self.focused_column = self.focused_column.decrement();
                self.reset_focused_task(context);
                None
            }
            KeyCode::Right => {
                self.focused_column = self.focused_column.increment();
                self.reset_focused_task(context);
                None
            }
            KeyCode::BackTab => {
                self.focused_tab = self.focused_tab.decrement();
                self.reset_focused_column(context);
                self.focused_task =
                    ListState::new(self.get_current_column_len(context).checked_sub(1));
                None
            }
            KeyCode::Tab => {
                self.focused_tab = self.focused_tab.increment();
                self.reset_focused_column(context);
                None
            }
            KeyCode::Esc => {
                switch_to_view(MainView::new(context.state.projects().len().checked_sub(1)))
            }
            KeyCode::Enter => self.focused_task.focused_item().and_then(|_| {
                open_prompt(MoveToColumnPrompt::new(
                    context,
                    self.get_current_column(context.config).name.clone(),
                ))
            }),
            _ if !self.get_current_column(context.config).done_column => {
                self.on_key_mutable(key_event, context)
            }
            _ => self.focused_task.on_key(key_event, context),
        }
    }

    fn key_hints(&self, context: Context) -> KeyHints {
        vec![
            ("Tab/Backtab", "Switch between tabs"),
            ("Left/Right", "Switch between columns"),
            ("Esc", "Back to projects view"),
            ("Enter", "Move task to column"),
        ]
        .tap_mut(|base| {
            if !self.get_current_column(context.config).done_column {
                base.extend([
                    ("Delete/Backspace", "Delete"),
                    ("n", "New"),
                    ("p", "Set priority"),
                    ("d", "Set difficulty"),
                    ("r", "Rename"),
                    ("a", "Add due date"),
                ]);
            }
        })
    }

    fn render(&self, area: Rect, buf: &mut Buffer, context: Context) {
        let tab_constraints = (0..context.config.tabs.len()).map(|tab| {
            if !context.config.collapse_unfocused_tabs || tab == usize::from(self.focused_tab) {
                Constraint::Min(0)
            } else {
                Constraint::Length(1)
            }
        });
        Layout::vertical(tab_constraints)
            .split(area)
            .iter()
            .enumerate()
            .for_each(|(tab, area)| self.render_tab(*area, buf, context, tab));
    }
}
