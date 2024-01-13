# Use hurl to test the mock server defined in tour.hurl.

GET http://localhost:3939/
HTTP 200
`hello, world!`

POST http://localhost:3939/
HTTP 201
Content-Type: application/json
{"id": 1}

PUT http://localhost:3939/
HTTP 200
`updated`

DELETE http://localhost:3939/
HTTP 204

GET http://localhost:3939/users?name=John
HTTP 200
{"name": "John"}

GET http://localhost:3939/users
X-User-Name: John
HTTP 200
{"name": "John"}

GET http://localhost:3939/users
[Cookies]
user_name: John
HTTP 200
{"name": "John"}

GET http://localhost:3939/users?name=Jane
HTTP 200
{"name": "Jane"}

GET http://localhost:3939/is_authorized?user_name=John
HTTP 200
{"is_authorized": true}

GET http://localhost:3939/is_authorized?user_name=Jonathan
HTTP 200
{"is_authorized": true}

GET http://localhost:3939/is_authorized?user_name=Alice
HTTP 200
{"is_authorized": false}

GET http://localhost:3939/better_is_authorized?user_name=John
HTTP 200
{"is_authorized": true}

GET http://localhost:3939/better_is_authorized?user_name=Jon
HTTP 200
{"is_authorized": false}

GET http://localhost:3939/better_is_authorized?user_name=Jane
HTTP 200
{"is_authorized": false}

GET http://localhost:3939/offers?id=1
HTTP 200
{"name": "Offer 1"}

GET http://localhost:3939/offers?id=3
HTTP 404
{"error": "Offer not found"}

GET http://localhost:3939/offers
HTTP 400
{"error": "Expected offer id"}

GET http://localhost:3939/orders?id=123
Accept: application/json
HTTP 200
Content-Type: application/json
{"id": 123}

GET http://localhost:3939/orders?id=123
Accept: text/html
HTTP 200
Content-Type: text/html
`<html><body>Order 123</body></html>`

GET http://localhost:3939/__route_that_does_not_exist__
HTTP 404
