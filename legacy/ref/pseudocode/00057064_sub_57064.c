/* ea:   0x00057064 */
/* name: sub_57064 */
/* size: 64 */
int __fastcall sub_57064(unsigned __int16 *a1)
{
  _DWORD *v2; // r1
  unsigned int v3; // r0
  int result; // r0

  v2 = a1 + 4;
  v3 = *((_DWORD *)a1 + 2);
  v2[1] = 999999;
  if ( v3 > 0xF423F )
    *v2 = 999999;
  result = *a1;
  a1[1] = 999;
  if ( (unsigned __int16)result > 0x3E7u )
    *a1 = 999;
  return result;
}

