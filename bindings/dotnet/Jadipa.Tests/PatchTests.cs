namespace Jadipa.Tests;

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

        var result = Jadipa.ApplyPatchJson(target, patch);

        Assert.Contains("\"name\":\"new\"", result);
        Assert.Contains("\"items\":[1,2,3]", result);
    }

    [Fact]
    public void ApplyPatchJson_InvalidJson_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Jadipa.ApplyPatchJson("{", "[]"));

        Assert.Contains("JSON", ex.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ApplyPatchJson_InvalidPatch_ThrowsWithDescription()
    {
        var ex = Assert.Throws<JadipaErrorException>(() =>
            Jadipa.ApplyPatchJson("""{"name":"old"}""", """{"op":"replace"}"""));

        Assert.NotEmpty(ex.Message);
    }
}
