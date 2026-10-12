/* ea:   0x000314C4 */
/* name: sub_314C4 */
/* size: 64 */
int __fastcall sub_314C4(int a1)
{
  int result; // r0

  if ( (unsigned int)(*(unsigned __int8 *)(*(_DWORD *)(a1 + 12) + 22) << 26) >> 28 != 0
    || (LOBYTE(result) = 1, *(_BYTE *)(*(_DWORD *)(a1 + 20) + 50) != 0) )
  {
    LOBYTE(result) = 0;
  }
  return (unsigned __int8)result;
}

