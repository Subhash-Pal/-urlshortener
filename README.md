# API Request 
#Test under Postman or any API Client

Input : 

#PUT  127.0.0.1:8080/send-url  { "url": "https://C.wand.ai/saved-wandys?tab=llm-zoo" } // To send URL

Output: {

    "original_url": "https://C.wand.ai/saved-wandys?tab=llm-zoo",
    
    "shortened_url": "715f7690f235",
    
    "request_count": 6
    
}
 

#GET 127.0.0.1:8080/715f7690f235  //{715f7690f235} "shorten URL Hash code"  Will return Orignal URL 

#GET Request => 127.0.0.1:8080/metrics                                                 // To get top 3 URL requested 

Output : [

    {
        "domain": "C.wand.ai",
        
        "count": 6
    }
]
