/* ea:   0x00052998 */
/* name: sub_52998 */
/* size: 60 */
BOOL __fastcall sub_52998(_DWORD *a1, unsigned int a2)
{
  char v3; // r4

  v3 = a2;
  if ( a2 >= 8 )
    TerminateProcess((HANDLE)0x42, 0);
  return (*a1 & (1 << (v3 & 0x1F))) != 0;
}

