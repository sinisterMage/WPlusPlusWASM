using System.Text.Json;
using WPlusPlus;
using WPlusPlus.AST;

var code = File.ReadAllText("../ui.wpp");
var tokens = Lexer.Tokenize(code);
var parser = new Parser(tokens);

var ast = parser.Parse();

// Collect BoxNodes only
var boxes = new List<BoxNode>();
if (ast is BlockNode block)
{
    foreach (var node in block.Statements)
    {
        if (node is BoxNode box)
            boxes.Add(box);
    }
}

// Export to JSON
var json = JsonSerializer.Serialize(boxes, new JsonSerializerOptions { WriteIndented = true });
File.WriteAllText("../out.box.json", json);

Console.WriteLine("✅ out.box.json written with " + boxes.Count + " boxes.");
