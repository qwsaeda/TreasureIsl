/* ea:   0x00011634 */
/* name: sub_11634 */
/* size: 60 */
int __fastcall sub_11634(int a1, int a2, int a3, int (__fastcall *a4)(int))
{
  int v6; // r5
  int result; // r0
  int i; // r4

  v6 = a1 + a2 * a3;
  result = a3 - 1;
  if ( a3 - 1 >= 0 )
  {
    for ( i = a3; i != 0; --i )
    {
      v6 -= a2;
      result = a4(v6);
    }
  }
  return result;
}

