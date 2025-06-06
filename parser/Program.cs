using System.Text.Json;
using WPlusPlus;
using WPlusPlus.AST;

var code = File.ReadAllText("../ui.wpp");
var tokens = Lexer.Tokenize(code);
var parser = new Parser(tokens);
var ast = parser.Parse();

var json = JsonSerializer.Serialize(ast, new JsonSerializerOptions
{
    WriteIndented = true,
    Converters = { new NodeJsonConverter() }
});

File.WriteAllText("../out.box.json", json);
Console.WriteLine("✅ out.box.json written with semantic W++ layout");
