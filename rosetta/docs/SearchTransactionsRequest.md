# SearchTransactionsRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**network_identifier** | [**crate::models::NetworkIdentifier**](NetworkIdentifier.md) |  | 
**operator** | Option<[**crate::models::Operator**](Operator.md)> |  | [optional]
**max_block** | Option<**i64**> | max_block is the largest block index to consider when searching for transactions. If this field is not populated, the current block is considered the max_block.  If you do not specify a max_block, it is possible a newly synced block will interfere with paginated transaction queries (as the offset could become invalid with newly added rows).  | [optional]
**offset** | Option<**i64**> | offset is the offset into the query result to start returning transactions.  If any search conditions are changed, the query offset will change and you must restart your search iteration.  | [optional]
**limit** | Option<**i64**> | limit is the maximum number of transactions to return in one call. The implementation may return <= limit transactions.  | [optional]
**transaction_identifier** | Option<[**crate::models::TransactionIdentifier**](TransactionIdentifier.md)> |  | [optional]
**account_identifier** | Option<[**crate::models::AccountIdentifier**](AccountIdentifier.md)> |  | [optional]
**coin_identifier** | Option<[**crate::models::CoinIdentifier**](CoinIdentifier.md)> |  | [optional]
**currency** | Option<[**crate::models::Currency**](Currency.md)> |  | [optional]
**status** | Option<**String**> | status is the network-specific operation type.  | [optional]
**_type** | Option<**String**> | type is the network-specific operation type.  | [optional]
**address** | Option<**String**> | address is AccountIdentifier.Address. This is used to get all transactions related to an AccountIdentifier.Address, regardless of SubAccountIdentifier.  | [optional]
**success** | Option<**bool**> | success is a synthetic condition populated by parsing network-specific operation statuses (using the mapping provided in `/network/options`).  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


