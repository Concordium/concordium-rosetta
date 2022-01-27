# Operation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operation_identifier** | [**crate::models::OperationIdentifier**](OperationIdentifier.md) |  | 
**related_operations** | Option<[**Vec<crate::models::OperationIdentifier>**](OperationIdentifier.md)> | Restrict referenced related_operations to identifier indices < the current operation_identifier.index. This ensures there exists a clear DAG-structure of relations.  Since operations are one-sided, one could imagine relating operations in a single transfer or linking operations in a call tree.  | [optional]
**_type** | **String** | Type is the network-specific type of the operation. Ensure that any type that can be returned here is also specified in the NetworkOptionsResponse. This can be very useful to downstream consumers that parse all block data.  | 
**status** | Option<**String**> | Status is the network-specific status of the operation. Status is not defined on the transaction object because blockchains with smart contracts may have transactions that partially apply (some operations are successful and some are not). Blockchains with atomic transactions (all operations succeed or all operations fail) will have the same status for each operation.  On-chain operations (operations retrieved in the `/block` and `/block/transaction` endpoints) MUST have a populated status field (anything on-chain must have succeeded or failed). However, operations provided during transaction construction (often times called \"intent\" in the documentation) MUST NOT have a populated status field (operations yet to be included on-chain have not yet succeeded or failed).  | [optional]
**account** | Option<[**crate::models::AccountIdentifier**](AccountIdentifier.md)> |  | [optional]
**amount** | Option<[**crate::models::Amount**](Amount.md)> |  | [optional]
**coin_change** | Option<[**crate::models::CoinChange**](CoinChange.md)> |  | [optional]
**metadata** | Option<[**serde_json::Value**](.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


