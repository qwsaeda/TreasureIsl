/* ea:   0x0004A8F0 */
/* name: sub_4A8F0 */
/* size: 68 */
int __fastcall sub_4A8F0(_DWORD *a1)
{
  int result; // r0

  if ( (unsigned int)(*(unsigned __int8 *)(a1[3] + 22) << 26) >> 28 != 0
    || (LOBYTE(result) = 1, *(unsigned __int8 *)(a1[5] + 50) != a1[2]) )
  {
    LOBYTE(result) = 0;
  }
  return (unsigned __int8)result;
}

