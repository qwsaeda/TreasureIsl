/* ea:   0x00011478 */
/* name: sub_11478 */
/* size: 92 */
int __fastcall sub_11478(int a1, HANDLE hFile)
{
  int v2; // r0
  DWORD Buffer[4]; // [sp+4h] [bp-10h] BYREF

  if ( hFile == (HANDLE)-1 )
    return 0;
  v2 = *(_DWORD *)(a1 + 36);
  if ( v2 != 0 )
  {
    Buffer[0] = *(unsigned __int8 *)(v2 + 50);
    WriteFile(hFile, Buffer, 4u, &Buffer[1], nullptr);
  }
  return 1;
}

