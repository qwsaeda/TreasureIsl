/* ea:   0x0001B00C */
/* name: sub_1B00C */
/* size: 20 */
void __fastcall sub_1B00C(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

