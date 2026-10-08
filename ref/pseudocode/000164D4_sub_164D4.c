/* ea:   0x000164D4 */
/* name: sub_164D4 */
/* size: 44 */
int __fastcall sub_164D4(unsigned __int16 *a1, int a2)
{
  unsigned __int16 v3; // r0
  unsigned int v4; // r2
  int result; // r0

  v3 = *a1;
  a1[1] = a2;
  v4 = v3;
  result = a2 << 16;
  if ( v4 > (unsigned __int16)a2 )
    *a1 = a2;
  return result;
}

