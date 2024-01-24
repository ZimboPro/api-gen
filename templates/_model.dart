part "{{file_name | replace(from=".dart", to=".g.dart")}}";

@JsonSerializable()
class {{object_name}} {
    {% for field in properties -%}
    {% if field.property_type == "Array" %}final List<{{field.object_name}}>? {{field.name}};{% elif field.property_type == "Object" %}final {{field.object_name}}? {{field.name}};
    {% else %}final {{map_type(type = field )}}? {{field.name}};{% endif %}
    {% endfor %}
    {{object_name}}(
        {%- if properties -%}
        { {% for field in properties -%}{% if field.required %}
        required this.{{field.name}},{% else %}
        this.{{field.name}},{% endif %}{% endfor %}
    }
    {%- endif -%}
    );

    factory {{object_name}}.fromJson(Map<String, dynamic> json) => _${{object_name}}FromJson(json);

    Map<String, dynamic> toJson() => _${{object_name}}ToJson(this);
}


