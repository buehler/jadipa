namespace Jadipa.Tests;

using System.Text.Json.Nodes;

public class PatchTests
{
    [Fact]
    public void ApplyPatchJson_ReturnsPatchedJson()
    {
        var target = """{"name":"old","items":[1,2]}""";

        var patch = """
        [
          { "op": "replace", "path": "/name", "value": "new" },
          { "op": "add", "path": "/items/-", "value": 3 }
        ]
        """;

        var result = Patch.ApplyJson(target, patch);

        Assert.Contains("\"name\":\"new\"", result);
        Assert.Contains("\"items\":[1,2,3]", result);
    }

    [Fact]
    public void ApplyPatchJson_InvalidJson_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Patch.ApplyJson("{", "[]"));

        Assert.Contains("JSON", ex.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ApplyPatchJson_InvalidPatch_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Patch.ApplyJson("""{"name":"old"}""", """{"op":"replace"}"""));

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
