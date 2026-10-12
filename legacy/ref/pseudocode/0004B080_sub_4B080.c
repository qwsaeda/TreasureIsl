/* ea:   0x0004B080 */
/* name: sub_4B080 */
/* size: 28 */
int __fastcall sub_4B080(int *a1, int *a2)
{
  int result; // r0
  int v5; // r1

  result = *a2;
  v5 = *a1;
  *a1 = result;
  *a2 = v5;
  return result;
}

