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

GET http://localhost:3939/users?name=Jane
HTTP 404

GET http://localhost:3939/users
HTTP 404

GET http://localhost:3939/orders?id=123
Accept: application/json
HTTP 200
Content-Type: application/json
{"id": 123}

GET http://localhost:3939/orders?id=123
Accept: text/html
HTTP 200
Content-Type: text/html
<html><body>Order 123</body></html>

GET http://localhost:3939/offers?id=1
HTTP 200
Content-Type: application/json
{"id": 1, "name": "Offer 1"}

GET http://localhost:3939/offers?id=2
HTTP 200
Content-Type: application/json
{"id": 2, "name": "Offer 2"}

GET http://localhost:3939/offers?id=3
HTTP 404
Content-Type: application/json
{"error": "Offer not found"}

GET http://localhost:3939/offers
HTTP 400
Content-Type: application/json
{"error": "Offer ID must be present"}

GET http://localhost:3939/__route_that_does_not_exist__
HTTP 404
