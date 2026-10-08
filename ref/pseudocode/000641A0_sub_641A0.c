/* ea:   0x000641A0 */
/* name: sub_641A0 */
/* size: 72 */
int __fastcall sub_641A0(int *a1, int *a2)
{
  int v4; // r0
  int v5; // r1
  int v6; // r1
  int v7; // r1
  int result; // r0

  v4 = *a2;
  v5 = *a1;
  *a1 = v4;
  *a2 = v5;
  v6 = a1[1];
  a1[1] = a2[1];
  a2[1] = v6;
  v7 = a1[2];
  result = a2[2];
  a1[2] = result;
  a2[2] = v7;
  return result;
}

