
export interface Peer {
  username: string;
  node_id: string;
}
export interface TreeNode {
  name: string;
  type: 'file' | 'folder';
  children?: TreeNode[];
  path?: string;
  level?: number;
  size?: number;
  modified?: Date;
}

export interface FlatNode extends TreeNode {
  id: string;
  parentPath: string;
  level: number;
  isExpanded: boolean;
}

export interface DirectoryTreeProps {
  data: TreeNode[];
  onSelect?: (node: TreeNode) => void;
  searchable?: boolean;
  paginated?: boolean;
}

