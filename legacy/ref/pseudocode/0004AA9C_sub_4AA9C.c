/* ea:   0x0004AA9C */
/* name: sub_4AA9C */
/* size: 112 */
int __fastcall sub_4AA9C(int a1, unsigned int *a2)
{
  unsigned int v3; // r3
  int result; // r0

  v3 = ((unsigned int)(*(unsigned __int16 *)(*(_DWORD *)(a1 + 12) + 24) << 24) >> 30) + 1;
  *a2 = v3;
  if ( (unsigned int)(*(unsigned __int8 *)(*(_DWORD *)(a1 + 12) + 22) << 26) >> 28 != 1
    || (LOBYTE(result) = 1, v3 == *(_DWORD *)(a1 + 8)) )
  {
    LOBYTE(result) = 0;
  }
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
    *a2 = 0;
  return result;
}

