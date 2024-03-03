# API Request 
#Test under Postman or any API Client
# To Send new URL 

Input : 

#PUT  127.0.0.1:8080/shorten-and-retrieve-url  { "url": "https://C.wand.ai/saved-wandys?tab=llm-zoo" } // To send URL

{
    "original_url_received": "https://coderprog.com",
    "shortened_url": "844c01eb2e56",
    "original_url_retrieved": "https://coderprog.com",
    "original_url_matches": true,
    "received_count": 3
}
 
# To get orignal URL back from shortner hashcode 

#POST 127.0.0.1:8080/retrieve-original-url  

Body { "url": "844c01eb2e56" }          "shorten URL Hash code"  Will return Orignal URL 

Output : 
    "original_url_received": "https://coderprog.com",
    "shortened_url": "844c01eb2e56",
    "original_url_retrieved": "https://coderprog.com",
    "original_url_matches": true,
    "received_count": 4
}

#To get top 3 requested URL 

#GET 127.0.0.1:8080/top-urls                             // To get top 3 URL requested 

Output : [
    [
        "https://Z.coderprog.com",
        15
    ],
    [
        "https://C.coderprog.com",
        10
    ],
    [
        "https://coderprog.com",
        5
    ]
]
