/* ea:   0x0005710C */
/* name: sub_5710C */
/* size: 76 */
int __fastcall sub_5710C(_DWORD *a1)
{
  int result; // r0

  if ( (unsigned int)(*(unsigned __int8 *)(a1[3] + 22) << 26) >> 28 != 0
    || *(unsigned __int8 *)(a1[5] + 50) == a1[54]
    || (LOBYTE(result) = 1, *(_BYTE *)(a1[5] + 50) == 0) )
  {
    LOBYTE(result) = 0;
  }
  return (unsigned __int8)result;
}

