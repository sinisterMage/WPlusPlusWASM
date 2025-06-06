using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace WPlusPlus.AST
{
    public class NodeJsonConverter : JsonConverter<Node>
    {
        public override Node Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
            => throw new NotImplementedException("Deserialization not supported.");

        public override void Write(Utf8JsonWriter writer, Node value, JsonSerializerOptions options)
        {
            switch (value)
            {
                case BoxNode box:
                    writer.WriteStartObject();
                    writer.WriteString("Kind", "Box");
                    writer.WriteNumber("X", box.X);
                    writer.WriteNumber("Y", box.Y);
                    writer.WriteNumber("Width", box.Width);
                    writer.WriteNumber("Height", box.Height);
                    writer.WriteEndObject();
                    break;

                case GroupNode group:
                    writer.WriteStartObject();
                    writer.WriteString("Kind", "Group");
                    writer.WritePropertyName("Children");
                    JsonSerializer.Serialize(writer, group.Children, options);
                    writer.WriteEndObject();
                    break;

                case BlockNode block:
                    JsonSerializer.Serialize(writer, block.Statements, options);
                    break;

                default:
                    throw new NotSupportedException($"Unhandled node type: {value.GetType().Name}");
            }
        }
    }
}
