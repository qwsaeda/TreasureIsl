/* ea:   0x00057274 */
/* name: sub_57274 */
/* size: 112 */
int __fastcall sub_57274(int a1, unsigned int *a2)
{
  unsigned int v3; // r3
  int result; // r0

  v3 = ((unsigned int)(*(unsigned __int16 *)(*(_DWORD *)(a1 + 12) + 24) << 24) >> 30) + 1;
  *a2 = v3;
  if ( (unsigned int)(*(unsigned __int8 *)(*(_DWORD *)(a1 + 12) + 22) << 26) >> 28 != 1
    || (LOBYTE(result) = 1, v3 == *(_DWORD *)(a1 + 216)) )
  {
    LOBYTE(result) = 0;
  }
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
    *a2 = 0;
  return result;
}

