/* ea:   0x00032E00 */
/* name: sub_32E00 */
/* size: 60 */
int __fastcall sub_32E00(int *a1)
{
  int v2; // r0

  v2 = rand();
  a1[2] = 24;
  *a1 = v2 % 6;
  return v2 % 6 + 1;
}

