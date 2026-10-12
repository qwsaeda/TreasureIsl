/* ea:   0x0003AA9C */
/* name: sub_3AA9C */
/* size: 112 */
int sub_3AA9C(int a1, _WORD *a2, int a3, ...)
{
  int v6; // r0
  int result; // r0
  int v8; // r0

  v6 = sub_12F04(a2);
  for ( result = sub_3B0C0(a1, a2, 0, v6); result != -1; result = sub_3B0C0(a1, a2, 0, v8) )
  {
    sub_3AFD8(a1, result, 2, a3);
    v8 = sub_12F04(a2);
  }
  return result;
}

