/* ea:   0x00061F98 */
/* name: sub_61F98 */
/* size: 280 */
int __fastcall sub_61F98(int a1, const WCHAR *a2, int a3, int a4)
{
  HANDLE FileForMappingW; // r0
  HANDLE FileMappingW; // r0
  LPVOID v8; // r0
  DWORD FileSize; // r0

  if ( a2 == nullptr )
    __debugbreak();
  if ( a3 != 0 || a4 != 0 )
    return 0;
  if ( (*(unsigned __int8 (__fastcall **)(int))(*(_DWORD *)a1 + 36))(a1) != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
  FileForMappingW = CreateFileForMappingW(a2, 0x80000000, 1u, nullptr, 3u, 0, nullptr);
  *(_DWORD *)(a1 + 4) = FileForMappingW;
  if ( FileForMappingW == (HANDLE)-1 )
    return 0;
  FileMappingW = CreateFileMappingW(FileForMappingW, nullptr, 2u, 0, 0, nullptr);
  *(_DWORD *)(a1 + 8) = FileMappingW;
  if ( FileMappingW == nullptr )
    return 0;
  v8 = MapViewOfFile(FileMappingW, 4u, 0, 0, 0);
  *(_DWORD *)(a1 + 12) = v8;
  if ( v8 == nullptr )
    return 0;
  FileSize = GetFileSize(*(HANDLE *)(a1 + 4), nullptr);
  *(_DWORD *)(a1 + 20) = 0;
  *(_DWORD *)(a1 + 16) = FileSize + *(_DWORD *)(a1 + 12);
  return 1;
}

