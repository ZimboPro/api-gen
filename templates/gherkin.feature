@{{extended(key="feature")}}Service
Feature: {{extended(key="feature")}} Service REST API

    Background:
        And def login = call read('file:src/test/java/com/olympus/services/features/customerServiceChat/login/login.feature')

    {% for endpoint in endpoints %}
    Scenario: {% if endpoint.description -%} {{endpoint.description}} {%- else -%} {{endpoint.path | camel_case}} {%- endif %}
        And def baseURL = Host
        Given url baseURL + '{{endpoint.path}}'
        And header Authorization = 'Bearer ' + login.access_token
        {% if endpoint.request -%}
        And request
            """
            {{json_value(structure = endpoint.request) | json_encode() | safe }}
            """
        {%- endif %}
        When method {{endpoint.method | upper}}
        Then status 200
        {# {{json_response(response = endpoint.response) | json_encode(pretty=true) | safe }} #}
        {% if endpoint.response -%}
        * def {{endpoint.response.object_name}} =
            """
            {{json_typing(structure = endpoint.response) | json_encode() | replace(from='["#string"]', to="#[] #string") | replace(from='["#number"]', to="#[] #number") | replace(from='["#boolean"]', to="#[] #boolean") | safe }}
            """
        And match response == {{endpoint.response.object_name}}
{%- endif %}
{% endfor %}

