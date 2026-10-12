/* ea:   0x000620B0 */
/* name: sub_620B0 */
/* size: 88 */
int __fastcall sub_620B0(int a1)
{
  int result; // r0

  if ( *(_DWORD *)(a1 + 8) != 0 )
  {
    UnmapViewOfFile(*(LPCVOID *)(a1 + 12));
    CloseHandle(*(HANDLE *)(a1 + 8));
    *(_DWORD *)(a1 + 8) = 0;
  }
  result = (*(int (__fastcall **)(int))(*(_DWORD *)a1 + 36))(a1);
  if ( (_BYTE)result != 0 )
  {
    result = CloseHandle(*(HANDLE *)(a1 + 4));
    *(_DWORD *)(a1 + 4) = 0;
  }
  return result;
}

