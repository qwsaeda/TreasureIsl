/* ea:   0x00048CA4 */
/* name: sub_48CA4 */
/* size: 56 */
int __fastcall sub_48CA4(int result, unsigned __int16 a2, char a3)
{
  unsigned int *v3; // r4
  unsigned int v4; // r1
  unsigned int v5; // r3

  if ( a3 != 0 )
  {
    v3 = (unsigned int *)(result + 144);
    result = *(_DWORD *)(result + 144);
    v4 = result + 1000 * a2;
    v5 = v3[1];
    *v3 = v4;
    if ( v4 > v5 )
      *v3 = v5;
  }
  return result;
}

