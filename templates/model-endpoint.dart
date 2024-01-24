part "{{file_name | replace(from=".dart", to=".g.dart")}}";

{%- for resp in models -%}
{% if resp.property_type == "Array" %}{% continue %}{% endif %}

@JsonSerializable()
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


