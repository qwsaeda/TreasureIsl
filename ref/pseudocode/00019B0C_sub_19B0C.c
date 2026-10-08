/* ea:   0x00019B0C */
/* name: sub_19B0C */
/* size: 44 */
int __fastcall sub_19B0C(int a1, int a2, __int16 **a3)
{
  int result; // r0

  if ( a2 != 0 )
    result = a1 + 36;
  else
    result = a1 + 20;
  if ( a3 != (__int16 **)result )
    return sub_17890(result, *a3, a3[1]);
  return result;
}

