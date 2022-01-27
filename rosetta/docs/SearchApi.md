# \SearchApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_transactions**](SearchApi.md#search_transactions) | **POST** /search/transactions | [INDEXER] Search for Transactions 



## search_transactions

> crate::models::SearchTransactionsResponse search_transactions(search_transactions_request)
[INDEXER] Search for Transactions 

`/search/transactions` allows the caller to search for transactions that meet certain conditions. Some conditions include matching a transaction hash, containing an operation with a certain status, or containing an operation that affects a certain account.  `/search/transactions` is considered an \"indexer\" endpoint and Rosetta implementations are not required to complete it to adhere to the Rosetta spec. However, any Rosetta \"indexer\" MUST support this endpoint. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_transactions_request** | [**SearchTransactionsRequest**](SearchTransactionsRequest.md) |  | [required] |

### Return type

[**crate::models::SearchTransactionsResponse**](SearchTransactionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

