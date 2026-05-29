namespace Jadipa.Tests;

using System.Text.Json.Nodes;

public class MergePatchTests
{
    [Fact]
    public void ApplyMergePatchJson_ReturnsPatchedJson()
    {
        var target = """
        {
          "title": "Goodbye!",
          "author": {
            "givenName": "John",
            "familyName": "Doe"
          },
          "tags": ["example", "sample"],
          "content": "This will be unchanged"
        }
        """;

        var patch = """
        {
          "title": "Hello!",
          "phoneNumber": "+01-123-456-7890",
          "author": {
            "familyName": null
          },
          "tags": ["example"]
        }
        """;

        var result = MergePatch.ApplyJson(target, patch);

        AssertJsonEqual("""
        {
          "title": "Hello!",
          "author": {
            "givenName": "John"
          },
          "tags": ["example"],
          "content": "This will be unchanged",
          "phoneNumber": "+01-123-456-7890"
        }
        """, result);
    }

    [Fact]
    public void ApplyMergePatchJson_InvalidPatch_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            MergePatch.ApplyJson("""{"name":"old"}""", """{"name":"new"""));

        Assert.NotEmpty(ex.Message);
    }

    private static void AssertJsonEqual(string expected, string actual)
    {
        var expectedJson = JsonNode.Parse(expected);
        var actualJson = JsonNode.Parse(actual);

        Assert.True(
            JsonNode.DeepEquals(expectedJson, actualJson),
            $"Expected JSON {expectedJson}, got {actualJson}");
    }
}
