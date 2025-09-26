use crate::DisplayValueTypeView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_values::display_value_type::DisplayValueType;

pub struct DisplayValueTypeConverter {}

impl DisplayValueTypeConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DisplayValueType, DisplayValueTypeView> for DisplayValueTypeConverter {
    fn convert_collection(
        &self,
        display_value_list: &Vec<DisplayValueType>,
    ) -> Vec<DisplayValueTypeView> {
        display_value_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        display_value_type: &DisplayValueType,
    ) -> DisplayValueTypeView {
        match display_value_type {
            DisplayValueType::Bool => DisplayValueTypeView::Bool,
            DisplayValueType::String => DisplayValueTypeView::String,
            DisplayValueType::Binary => DisplayValueTypeView::Binary,
            DisplayValueType::Decimal => DisplayValueTypeView::Decimal,
            DisplayValueType::Hexadecimal => DisplayValueTypeView::Hexadecimal,
            DisplayValueType::Address => DisplayValueTypeView::Address,
            DisplayValueType::DataTypeRef => DisplayValueTypeView::DataTypeRef,
            DisplayValueType::Enumeration => DisplayValueTypeView::Enumeration,
        }
    }
}

impl ConvertFromViewData<DisplayValueType, DisplayValueTypeView> for DisplayValueTypeConverter {
    fn convert_from_view_data(
        &self,
        display_value_type: &DisplayValueTypeView,
    ) -> DisplayValueType {
        match display_value_type {
            DisplayValueTypeView::Bool => DisplayValueType::Bool,
            DisplayValueTypeView::String => DisplayValueType::String,
            DisplayValueTypeView::Binary => DisplayValueType::Binary,
            DisplayValueTypeView::Decimal => DisplayValueType::Decimal,
            DisplayValueTypeView::Hexadecimal => DisplayValueType::Hexadecimal,
            DisplayValueTypeView::Address => DisplayValueType::Address,
            DisplayValueTypeView::DataTypeRef => DisplayValueType::DataTypeRef,
            DisplayValueTypeView::Enumeration => DisplayValueType::Enumeration,
        }
    }
}
