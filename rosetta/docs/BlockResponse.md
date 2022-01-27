# BlockResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**block** | Option<[**crate::models::Block**](Block.md)> |  | [optional]
**other_transactions** | Option<[**Vec<crate::models::TransactionIdentifier>**](TransactionIdentifier.md)> | Some blockchains may require additional transactions to be fetched that weren't returned in the block response (ex: block only returns transaction hashes). For blockchains with a lot of transactions in each block, this can be very useful as consumers can concurrently fetch all transactions returned.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


