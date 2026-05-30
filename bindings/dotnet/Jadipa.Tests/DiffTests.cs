namespace Jadipa.Tests;

using System.Text.Json.Nodes;

public class DiffTests
{
    [Fact]
    public void DiffJson_ReturnsPatchThatTransformsSourceJson()
    {
        var source = """
        {
          "name": "old",
          "tags": ["stable", "legacy"],
          "meta": {
            "enabled": false
          },
          "temporary": "remove-me"
        }
        """;

        var target = """
        {
          "name": "new",
          "tags": ["stable", "legacy", "dotnet"],
          "meta": {
            "enabled": true
          }
        }
        """;

        var patch = Diff.DiffJson(source, target);
        var result = Patch.ApplyJson(source, patch);

        AssertJsonEqual(target, result);
    }

    [Fact]
    public void DiffJson_EqualJson_ReturnsEmptyPatch()
    {
        var source = """{"name":"same","items":[1,2,3]}""";

        var patch = Diff.DiffJson(source, source);

        AssertJsonEqual("[]", patch);
    }

    [Fact]
    public void DiffJson_InvalidSourceJson_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Diff.DiffJson("{\"name\":\"old\"", "{\"name\":\"new\"}"));

        Assert.Contains("JSON", ex.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void DiffJson_InvalidTargetJson_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Diff.DiffJson("{\"name\":\"old\"}", "{\"name\":\"new\""));

        Assert.Contains("JSON", ex.Message, StringComparison.OrdinalIgnoreCase);
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
