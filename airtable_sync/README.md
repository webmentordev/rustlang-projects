## Information:
When you run this project, it will automatically sync all the records from Airtable, then wait for 7,200 seconds (2 hours) before retrying the sync. You can adjust the sync interval and the number of records processed per page. All records are fetched from Airtable first, then inserted or updated in the system. If a record already exists, it will be updated; otherwise, a new record will be created. You can use a column name, and its values will be converted to slugs so you can fetch the data. By default, if **ENABLE_FIELD_FILTERING** is set to true, only specific record fields, in **ALLOWED_FIELDS** will be sent in the API response. Otherwise, all fields will be included in the response for both single and multiple record fetches.
It is recommended to run this on a server with at least two threads.

## API Documentation
The record fetch limit is set to 300 by default, but you can modify it at compile time or at runtime using the following query parameters.
```
# Get first 300 records (page 1, default chunk size)
GET http://127.0.0.1:8080/records

# Get records 301-600 (page 2)
GET http://127.0.0.1:8080/records?page=2

# Get records 601-900 (page 3)
GET http://127.0.0.1:8080/records?page=3

# Custom page size (e.g., 100 records per page)
GET http://127.0.0.1:8080/records?page=1&page_size=100

# Get records 101-200 with custom page size
GET http://127.0.0.1:8080/records?page=2&page_size=400

# Get a record
GET http://127.0.0.1:8080/record/slug-of-the-record
```