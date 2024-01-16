import 'package:olympus/data/backend/api/http_service.dart';

class {{extended(key="feature")}}Service {
    HttpService __httpService;

    {{extended(key="feature")}}Service({required this.__httpService});

    {% for endpoint in endpoints %}
    {% if endpoint.response %}
    /// {{endpoint.description}}
    Future<
    {%- if endpoint.response.property_type == "Array" -%}
    List<{{endpoint.response.object_name}}>
    {%- elif endpoint.response.property_type == "Object" -%}
    {{endpoint.response.name}}
    {%- else -%}
    {{endpoint.response.property_type}}
    {%- endif -%}
    > {{endpoint.method | lower}}{{ endpoint.path | camel_case }}(
        {%- if endpoint.request -%}
        {{endpoint.request.name}} request
        {%- endif -%}
    ) async {
        final response = await _httpService
                .{{endpoint.method | lower}}DecodeSerialize<{%- if endpoint.response.property_type == "Array" -%}
    List<{{endpoint.response.object_name}}>
    {%- elif endpoint.response.property_type == "Object" -%}
    {{endpoint.response.name}}
    {%- else -%}
    {{endpoint.response.property_type}}
    {%- endif -%}>(
                        '{{endpoint.path}}', (p0) { // TODO map version to ${ApiVersionHelper.apiVersionV1}
            return {{endpoint.response.name}}FromJson(p0);
        }
        {%- if endpoint.request -%}
        , data: request.toJson()
        {%- endif -%});
        return response;
    }
    {%- else -%}
    /// {{endpoint.description}}
    Future<void> {{endpoint.method | lower}}{{ endpoint.path | camel_case }}(
        {%- if endpoint.request -%}
        {{endpoint.request.name}} request
        {%- endif -%}
    ) async {
        final response = await _httpService
                .{{endpoint.method | lower}}(
                        '{{endpoint.path}}', (p0) => null
        {%- if endpoint.request -%}
        , data: request.toJson()
        {%- endif -%});
        return response;
    }
    {% endif %}
    {% endfor %}

    {%- for response in responses -%}
    {%if response.is_root == false %}{% continue %}{% endif %}
    static {{response.name}} {{response.name}}FromJson(Map<String, dynamic> json) =>
            {{response.name}}.fromJson(json);
    {% endfor %}
}

// Request classes

{% for resp in requests -%}
{% if resp.property_type == "Array" %}{% continue %}{% endif %}

class {{resp.object_name}} {
    {% for field in resp.properties -%}
    {% if field.property_type == "Array" %}final List<{{field.object_name}}>? {{field.name}};{% elif field.property_type == "Object" %}final {{field.object_name}}? {{field.name}};
    {% else %}final {{map_type(type = field )}}? {{field.name}};{% endif %}
    {% endfor %}
    {{resp.object_name}}({ {% for field in resp.properties -%}{% if field.required %}
        required this.{{field.name}},{% else %}
        this.{{field.name}},{% endif %}{% endfor %}
    });

    factory {{resp.object_name}}.fromJson(Map<String, dynamic> json) => _${{resp.object_name}}FromJson(json);

    Map<String, dynamic> toJson() => _${{resp.object_name}}ToJson(this);
}

{%- endfor %}

// Response classes

{%- for resp in responses -%}
{% if resp.property_type == "Array" %}{% continue %}{% endif %}

class {{resp.object_name}} {
    {% for field in resp.properties -%}
    {% if field.property_type == "Array" %}final List<{{field.object_name}}>? {{field.name}};{% elif field.property_type == "Object" %}final {{field.object_name}}? {{field.name}};
    {% else %}final {{map_type(type = field)}}? {{field.name}};{% endif %}
    {% endfor %}
    {{resp.object_name}}({ {% for field in resp.properties -%}{% if field.required %}
        required this.{{field.name}},{% else %}
        this.{{field.name}},{% endif %}{% endfor %}
    });

    factory {{resp.object_name}}.fromJson(Map<String, dynamic> json) => _${{resp.object_name}}(json)

    Map<String, dynamic> toJson() => _${{resp.object_name}}ToJson(this);
}

{%- endfor %}
