/* ea:   0x00062F00 */
/* name: sub_62F00 */
/* size: 84 */
BOOL __fastcall sub_62F00(_DWORD *a1, LPVOID ptr, int a3)
{
  LPVOID v5; // r0

  v5 = CeZeroPointer(ptr);
  *a1 = -442564604;
  a1[1] = -442515452;
  a1[4] = v5;
  a1[2] = -509546484;
  a1[3] = a3;
  return FlushInstructionCache((HANDLE)0x42, a1, 0x14u);
}

