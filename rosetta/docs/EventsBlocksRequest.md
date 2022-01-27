# EventsBlocksRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**network_identifier** | [**crate::models::NetworkIdentifier**](NetworkIdentifier.md) |  | 
**offset** | Option<**i64**> | offset is the offset into the event stream to sync events from. If this field is not populated, we return the limit events backwards from tip. If this is set to 0, we start from the beginning.  | [optional]
**limit** | Option<**i64**> | limit is the maximum number of events to fetch in one call. The implementation may return <= limit events.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


