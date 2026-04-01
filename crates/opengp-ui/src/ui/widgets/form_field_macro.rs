macro_rules! impl_form_field_wrapper {
    ($field_enum:ty, $inner:ty) => {
        impl $field_enum {
            pub fn all() -> Vec<$field_enum> {
                <Self as crate::ui::widgets::FormField>::all()
            }

            pub fn label(&self) -> &'static str {
                <Self as crate::ui::widgets::FormField>::label(self)
            }

            pub fn id(&self) -> &'static str {
                <Self as crate::ui::widgets::FormField>::id(self)
            }

            pub fn from_id(id: &str) -> Option<Self> {
                <Self as crate::ui::widgets::FormField>::from_id(id)
            }

            pub fn is_required(&self) -> bool {
                <Self as crate::ui::widgets::FormField>::is_required(self)
            }

            pub fn is_textarea(&self) -> bool {
                <Self as crate::ui::widgets::FormField>::is_textarea(self)
            }

            pub fn is_dropdown(&self) -> bool {
                <Self as crate::ui::widgets::FormField>::is_dropdown(self)
            }
        }
    };
}

pub(crate) use impl_form_field_wrapper;
