/* ea:   0x000114D4 */
/* name: sub_114D4 */
/* size: 132 */
int __fastcall sub_114D4(int a1, HANDLE hFile, int a3)
{
  DWORD Buffer[6]; // [sp+4h] [bp-18h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  if ( *(_DWORD *)(a1 + 36) != 0 )
  {
    ReadFile(hFile, Buffer, 4u, &Buffer[1], nullptr);
    (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(a1 + 40) + 32))(
      *(_DWORD *)(a1 + 40),
      *(_DWORD *)(*(_DWORD *)(a3 + 32) + 4 * Buffer[0] + 60));
  }
  return 1;
}

