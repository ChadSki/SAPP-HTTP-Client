-- HTTP Client Example by 002 v1.1

api_version = "1.9.0.0"

ffi = require("ffi")
ffi.cdef [[
    typedef void http_response;
    http_response *http_get(const char *url, bool async);
    void http_destroy_response(http_response *);
    void http_wait_async(const http_response *);
    bool http_response_is_null(const http_response *);
    bool http_response_received(const http_response *);
    const char *http_read_response(const http_response *);
    uint32_t http_response_length(const http_response *);
]]
http_client = ffi.load("lua_http_client")

function GetPage(URL)
    local response = http_client.http_get(URL, false)
    local returning = nil
    if http_client.http_response_is_null(response) ~= true then
        local response_text_ptr = http_client.http_read_response(response)
        returning = ffi.string(response_text_ptr)
    end
    http_client.http_destroy_response(response)
    return returning
end

async_table = {}

function OnScriptLoad()
    cprint("Begin testing...")
    
    local ip_address_get = "https://api.ipify.org/"
    local random_number_get = "https://www.random.org/sequences/?min=1&max=10&num=1&col=10&format=plain&rnd=new"
    
    cprint("Test 1.1: Get IP Address from ipify.org...")
    local ip_address = GetPage(ip_address_get)
    if ip_address == nil then
        cprint("- (v)> Failed")
    else
        cprint("+ (^)> Passed: " .. ip_address)
    end
    
    cprint("Test 1.2: Get a random sequence between 1 and 10 from random.org...")
    local random_number = GetPage(random_number_get)
    if random_number == nil then
        cprint("- (v)> Failed")
    else
        cprint("+ (^)> Passed: " .. string.gsub(string.gsub(random_number,"\t"," "),"\n",""))
    end
    
    cprint("Test 2: Do the previous two tests asynchronously...")
    async_table["IP Address"] = http_client.http_get(ip_address_get, true)
    timer(1,"CheckResult","IP Address",os.clock())
    async_table["Random Number"] = http_client.http_get(random_number_get, true)
    timer(1,"CheckResult","Random Number",os.clock())
end

ceil = math.ceil

function CheckResult(ResultName,Time)
    if http_client.http_response_received(async_table[ResultName]) then
        if http_client.http_response_is_null(async_table[ResultName]) then
            cprint("- (v)> Failed to get " .. ResultName .. " (" .. ceil((os.clock() - tonumber(Time)) * 1000) .. "ms" .. ")")
        else
            cprint("+ (^)> Passed: " .. ResultName .. ": " .. string.gsub(string.gsub(ffi.string(http_client.http_read_response(async_table[ResultName])),"\t"," "),"\n","") .. " (Received in " .. ceil((os.clock() - tonumber(Time)) * 1000) .. " ms" .. ")")
        end
        http_client.http_destroy_response(async_table[ResultName])
        return false
    end
    return true
end

function OnScriptUnload()end