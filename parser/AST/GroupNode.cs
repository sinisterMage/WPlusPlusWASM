using System.Collections.Generic;

namespace WPlusPlus.AST
{
    public class GroupNode : Node
    {
        public List<Node> Children { get; set; }

        public GroupNode(List<Node> children)
        {
            Children = children;
        }
    }
}
