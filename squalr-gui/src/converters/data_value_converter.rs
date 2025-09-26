use crate::DataValueViewData;
use crate::converters::data_type_ref_converter::DataTypeRefConverter;
use crate::converters::display_values_converter::DisplayValuesConverter;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

pub struct DataValueConverter {}

impl DataValueConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<DataValue, DataValueViewData> for DataValueConverter {
    fn convert_collection(
        &self,
        data_value_list: &Vec<DataValue>,
    ) -> Vec<DataValueViewData> {
        data_value_list
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        data_value: &DataValue,
    ) -> DataValueViewData {
        DataValueViewData {
            data_type_ref: DataTypeRefConverter {}.convert_to_view_data(data_value.get_data_type_ref()),
            display_values: DisplayValuesConverter {}.convert_to_view_data(data_value.get_display_values()),
        }
    }
}
