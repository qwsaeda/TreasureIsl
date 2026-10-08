/* ea:   0x000309A0 */
/* name: sub_309A0 */
/* size: 32 */
int __fastcall sub_309A0(int a1, int a2)
{
  int result; // r0
  unsigned int *v3; // r3
  unsigned int v4; // r2
  unsigned int v5; // r1

  v3 = (unsigned int *)(a1 + 8);
  result = *(_DWORD *)(a1 + 8);
  v4 = v3[1];
  v5 = result + a2;
  *v3 = v5;
  if ( v5 > v4 )
    *v3 = v4;
  return result;
}

